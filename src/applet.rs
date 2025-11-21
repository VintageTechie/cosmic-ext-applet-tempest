// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Task, Core};
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Limits, Subscription};
use cosmic::iced_futures::Subscription as IcedSubscription;
use cosmic::widget::{self, text, settings};
use cosmic::{Application, Element, Action};
use std::time::Duration;

use crate::config::{Config, TemperatureUnit};
use crate::weather::{WeatherData, fetch_weather, weathercode_to_description, weathercode_to_icon_name, format_hour, format_time, detect_location, search_city, LocationResult};

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct Tempest {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<Id>,
    /// Weather data.
    weather_data: Option<WeatherData>,
    /// Configuration
    config: Config,
    /// Config handler for persistence
    config_handler: Option<cosmic::cosmic_config::Config>,
    /// Input field states
    city_input: String,
    refresh_input: String,
    /// Search results
    search_results: Vec<LocationResult>,
    /// Display label for panel button
    display_label: String,
    /// Current weather code for icon display
    current_weathercode: i32,
    /// Loading state
    is_loading: bool,
    /// Error state
    error_message: Option<String>,
    /// Collapsible section states
    hourly_expanded: bool,
    forecast_expanded: bool,
    settings_expanded: bool,
}

impl Default for Tempest {
    fn default() -> Self {
        let config = Config::default();
        Self {
            core: Default::default(),
            popup: None,
            weather_data: None,
            city_input: String::new(),
            refresh_input: config.refresh_interval_minutes.to_string(),
            search_results: Vec::new(),
            display_label: "...".to_string(),
            current_weathercode: 0,
            is_loading: true,
            error_message: None,
            hourly_expanded: true,
            forecast_expanded: true,
            settings_expanded: true,
            config,
            config_handler: None,
        }
    }
}

/// This is the enum that contains all the possible variants that your application will need to transmit messages.
/// This is used to communicate between the different parts of your application.
/// If your application does not need to send messages, you can use an empty enum or `()`.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    RefreshWeather,
    WeatherUpdated(Result<WeatherData, String>),
    Tick,
    ToggleTemperatureUnit,
    UpdateCityInput(String),
    SearchCity,
    CitySearchResult(Result<Vec<LocationResult>, String>),
    SelectLocation(usize),
    UpdateRefreshInterval(String),
    DetectLocation,
    LocationDetected(Result<(f64, f64, String), String>),
    ToggleAutoLocation,
    ToggleHourly,
    ToggleForecast,
    ToggleSettings,
}

/// Implement the `Application` trait for your application.
/// This is where you define the behavior of your application.
///
/// The `Application` trait requires you to define the following types and constants:
/// - `Executor` is the async executor that will be used to run your application's commands.
/// - `Flags` is the data that your application needs to use before it starts.
/// - `Message` is the enum that contains all the possible variants that your application will need to transmit messages.
/// - `APP_ID` is the unique identifier of your application.
impl Application for Tempest {
    type Executor = cosmic::executor::Default;

    type Flags = ();

    type Message = Message;

    const APP_ID: &'static str = "com.vintagetechie.CosmicExtAppletTempest";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    /// This is the entry point of your application, it is where you initialize your application.
    ///
    /// Any work that needs to be done before the application starts should be done here.
    ///
    /// - `core` is used to passed on for you by libcosmic to use in the core of your own application.
    /// - `flags` is used to pass in any data that your application needs to use before it starts.
    /// - `Task` type is used to send messages to your application. `Task::none()` can be used to send no messages to your application.
    fn init(core: Core, _flags: Self::Flags) -> (Self, Task<Self::Message>) {
        let config_handler = cosmic::cosmic_config::Config::new(Self::APP_ID, Config::VERSION).ok();
        let config = config_handler
            .as_ref()
            .and_then(|h| Config::get_entry(h).ok())
            .unwrap_or_default();

        let refresh_input = config.refresh_interval_minutes.to_string();

        let app = Tempest {
            core,
            config: config.clone(),
            config_handler,
            city_input: String::new(),
            refresh_input,
            search_results: Vec::new(),
            display_label: "...".to_string(),
            ..Default::default()
        };

        // Start with auto-location if enabled, otherwise fetch weather
        let task = if config.use_auto_location {
            Task::perform(
                async {
                    detect_location().await
                        .map_err(|e| e.to_string())
                },
                |result| Action::App(Message::LocationDetected(result)),
            )
        } else {
            Task::perform(
                async { Message::RefreshWeather },
                Action::App
            )
        };

        (app, task)
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let interval_minutes = self.config.refresh_interval_minutes;

        // Use the interval value as part of the ID so subscription restarts when it changes
        IcedSubscription::run_with_id(
            (std::any::TypeId::of::<Self>(), interval_minutes),
            async_stream::stream! {
                let interval = Duration::from_secs(interval_minutes * 60);
                loop {
                    tokio::time::sleep(interval).await;
                    yield Message::Tick;
                }
            }
        )
    }

    fn on_close_requested(&self, id: Id) -> Option<Message> {
        Some(Message::PopupClosed(id))
    }

    /// This is the main view of your application, it is the root of your widget tree.
    ///
    /// The `Element` type is used to represent the visual elements of your application,
    /// it has a `Message` associated with it, which dictates what type of message it can send.
    ///
    /// To get a better sense of which widgets are available, check out the `widget` module.
    fn view(&self) -> Element<'_, Self::Message> {
        use cosmic::iced::Alignment;
        use chrono::{Local, Timelike};

        // Determine if it's night time (6pm to 6am)
        let is_night = matches!(Local::now().hour(), 18..24 | 0..6);

        // Use error icon if there's an error, otherwise use weather icon
        let icon_name = if self.error_message.is_some() {
            "dialog-error-symbolic"
        } else {
            weathercode_to_icon_name(self.current_weathercode, is_night)
        };

        let icon = widget::icon::from_name(icon_name)
            .size(16)
            .symbolic(true);

        let temperature_text = text(&self.display_label);

        let data = if self.core.applet.is_horizontal() {
            Element::from(
                widget::row()
                    .push(icon)
                    .push(temperature_text)
                    .align_y(Alignment::Center)
                    .spacing(4),
            )
        } else {
            Element::from(
                widget::column()
                    .push(icon)
                    .push(temperature_text)
                    .align_x(Alignment::Center)
                    .spacing(4),
            )
        };

        let button = widget::button::custom(data)
            .class(cosmic::theme::Button::AppletIcon)
            .on_press(Message::TogglePopup);

        let tooltip_text = if let Some(ref error) = self.error_message {
            format!("Error: {}", error)
        } else if self.weather_data.is_some() {
            format!("{} - {}",
                weathercode_to_description(self.current_weathercode),
                self.config.location_name
            )
        } else {
            "Loading...".to_string()
        };

        let button_with_tooltip = widget::tooltip(
            button,
            text(tooltip_text),
            widget::tooltip::Position::Bottom
        );

        widget::autosize::autosize(button_with_tooltip, widget::Id::unique()).into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut column = widget::column().spacing(10).padding(10).max_width(450);

        // Show error message if there is one
        if let Some(ref error) = self.error_message {
            column = column.push(
                widget::container(
                    widget::column()
                        .spacing(10)
                        .push(widget::icon::from_name("dialog-error-symbolic").size(48))
                        .push(text("Failed to load weather").size(18))
                        .push(text(error).size(14))
                        .push(
                            widget::button::standard("Retry")
                                .on_press(Message::RefreshWeather)
                        )
                )
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill)
            );
        } else if self.is_loading {
            column = column.push(
                widget::container(
                    widget::column()
                        .spacing(10)
                        .push(text("Loading weather data...").size(18))
                )
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill)
            );
        } else if let Some(ref weather) = self.weather_data {
            // Current conditions
            column = column.push(
                widget::row()
                    .spacing(10)
                    .push(text(format!("{:.0}{}", weather.current.temperature, self.config.temperature_unit.symbol())).size(32))
                    .push(text(weathercode_to_description(weather.current.weathercode)))
            );

            // Additional current conditions
            column = column.push(
                widget::row()
                    .spacing(20)
                    .push(text(format!("Feels like: {:.0}{}", weather.current.feels_like, self.config.temperature_unit.symbol())).size(14))
                    .push(text(format!("Humidity: {}%", weather.current.humidity)).size(14))
            );

            column = column.push(
                text(format!("Wind: {:.1} mph", weather.current.windspeed)).size(14)
            );

            // Sunrise/Sunset for today
            if let Some(first_day) = weather.forecast.first() {
                column = column.push(
                    widget::row()
                        .spacing(20)
                        .push(text(format!("☀️ Sunrise: {}", format_time(&first_day.sunrise))).size(14))
                        .push(text(format!("🌙 Sunset: {}", format_time(&first_day.sunset))).size(14))
                );
            }

            column = column.push(widget::divider::horizontal::default());

            // Hourly forecast with collapsible header
            let hourly_arrow = if self.hourly_expanded { "▼" } else { "▶" };
            column = column.push(
                widget::button::text(format!("{} Next 12 Hours", hourly_arrow))
                    .on_press(Message::ToggleHourly)
                    .width(cosmic::iced::Length::Fill)
            );

            if self.hourly_expanded {
                for hour in &weather.hourly {
                    column = column.push(
                        widget::row()
                            .spacing(10)
                            .push(text(format_hour(&hour.time)).width(80))
                            .push(widget::icon::from_name(weathercode_to_icon_name(hour.weathercode, false)).size(16).symbolic(true))
                            .push(text(format!("{:.0}{}", hour.temperature, self.config.temperature_unit.symbol())).width(50))
                            .push(text(format!("💧 {}%", hour.precipitation_probability)).width(60))
                    );
                }
            }

            column = column.push(widget::divider::horizontal::default());

            // 7-day forecast with collapsible header
            let forecast_arrow = if self.forecast_expanded { "▼" } else { "▶" };
            column = column.push(
                widget::button::text(format!("{} 7-Day Forecast", forecast_arrow))
                    .on_press(Message::ToggleForecast)
                    .width(cosmic::iced::Length::Fill)
            );

            if self.forecast_expanded {
                for day in &weather.forecast {
                    column = column.push(
                        widget::row()
                            .spacing(10)
                            .push(text(&day.date).width(100))
                            .push(text(format!("{:.0}°", day.temp_max)).width(40))
                            .push(text(format!("{:.0}°", day.temp_min)).width(40))
                            .push(text(weathercode_to_description(day.weathercode)))
                    );
                }
            }

            column = column.push(widget::divider::horizontal::default());

            // Settings section with collapsible header
            let settings_arrow = if self.settings_expanded { "▼" } else { "▶" };
            column = column.push(
                widget::button::text(format!("{} Settings", settings_arrow))
                    .on_press(Message::ToggleSettings)
                    .width(cosmic::iced::Length::Fill)
            );

            if self.settings_expanded {
                column = column.push(
                    settings::item(
                        "Temperature Unit",
                        widget::button::standard(self.config.temperature_unit.to_string())
                            .on_press(Message::ToggleTemperatureUnit)
                    )
                );

                column = column.push(
                    settings::item(
                        "Auto-detect Location",
                        widget::row()
                            .spacing(10)
                            .push(widget::toggler(self.config.use_auto_location)
                                .on_toggle(|_| Message::ToggleAutoLocation))
                            .push(widget::button::standard("Detect Now")
                                .on_press(Message::DetectLocation))
                    )
                );

                column = column.push(
                    settings::item(
                        "Current Location",
                        text(&self.config.location_name)
                    )
                );

                if !self.config.use_auto_location {
                    column = column.push(text("Search Location").size(14));
                    column = column.push(
                        widget::row()
                            .spacing(10)
                            .padding([0, 20])
                            .push(widget::text_input("Enter city name...", &self.city_input)
                                .on_input(Message::UpdateCityInput)
                                .on_submit(|_| Message::SearchCity)
                                .width(cosmic::iced::Length::Fill))
                            .push(widget::button::standard("Search")
                                .on_press(Message::SearchCity))
                    );

                    if !self.search_results.is_empty() {
                        for (idx, result) in self.search_results.iter().enumerate() {
                            column = column.push(
                                widget::button::text(&result.display_name)
                                    .on_press(Message::SelectLocation(idx))
                                    .padding(8)
                                    .width(cosmic::iced::Length::Fill)
                            );
                        }
                    }
                }

                column = column.push(
                    settings::item(
                        "Refresh Interval (minutes)",
                        widget::text_input("Minutes", &self.refresh_input)
                            .on_input(Message::UpdateRefreshInterval)
                    )
                );
            }
        }

        let scrollable = widget::scrollable(column)
            .height(cosmic::iced::Length::Fill);

        self.core.applet.popup_container(scrollable).into()
    }

    /// Application messages are handled here. The application state can be modified based on
    /// what message was received. Tasks may be returned for asynchronous execution on a
    /// background thread managed by the application's executor.
    fn update(&mut self, message: Self::Message) -> Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                return if let Some(p) = self.popup.take() {
                    destroy_popup(p)
                } else {
                    let new_id = Id::unique();
                    self.popup.replace(new_id);
                    let mut popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );
                    popup_settings.positioner.size_limits = Limits::NONE
                        .max_width(450.0)
                        .min_width(400.0)
                        .min_height(600.0)
                        .max_height(800.0);
                    get_popup(popup_settings)
                }
            }
            Message::PopupClosed(id) => {
                if self.popup.as_ref() == Some(&id) {
                    self.popup = None;
                }
            }
            Message::RefreshWeather => {
                self.is_loading = true;
                self.error_message = None;

                let lat = self.config.latitude;
                let lon = self.config.longitude;
                let temp_unit = self.config.temperature_unit.api_param().to_string();
                return Task::perform(
                    async move {
                        fetch_weather(lat, lon, &temp_unit).await
                            .map_err(|e| e.to_string())
                    },
                    |result| Action::App(Message::WeatherUpdated(result)),
                );
            }
            Message::WeatherUpdated(result) => {
                self.is_loading = false;

                match result {
                    Ok(data) => {
                        self.current_weathercode = data.current.weathercode;
                        let temp = format!("{:.0}{}", data.current.temperature, self.config.temperature_unit.symbol());
                        self.display_label = temp;
                        self.weather_data = Some(data);
                        self.error_message = None;
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch weather: {}", e);
                        self.display_label = "ERR".to_string();
                        self.current_weathercode = 0;
                        self.error_message = Some(e);
                    }
                }
            }
            Message::Tick => {
                return Task::perform(
                    async { Message::RefreshWeather },
                    Action::App
                );
            }
            Message::ToggleTemperatureUnit => {
                self.config.temperature_unit = match self.config.temperature_unit {
                    TemperatureUnit::Fahrenheit => TemperatureUnit::Celsius,
                    TemperatureUnit::Celsius => TemperatureUnit::Fahrenheit,
                };
                self.save_config();
                return Task::perform(
                    async { Message::RefreshWeather },
                    Action::App
                );
            }
            Message::UpdateCityInput(value) => {
                self.city_input = value;
            }
            Message::SearchCity => {
                let city = self.city_input.clone();
                if !city.is_empty() {
                    return Task::perform(
                        async move {
                            search_city(&city).await
                                .map_err(|e| e.to_string())
                        },
                        |result| Action::App(Message::CitySearchResult(result)),
                    );
                }
            }
            Message::CitySearchResult(result) => {
                match result {
                    Ok(results) => {
                        self.search_results = results;
                    }
                    Err(e) => {
                        eprintln!("City search failed: {}", e);
                        self.search_results.clear();
                    }
                }
            }
            Message::SelectLocation(idx) => {
                if let Some(location) = self.search_results.get(idx) {
                    self.config.latitude = location.latitude;
                    self.config.longitude = location.longitude;
                    self.config.location_name = location.display_name.clone();
                    self.config.use_auto_location = false;
                    self.city_input.clear();
                    self.search_results.clear();
                    self.save_config();
                    return Task::perform(
                        async { Message::RefreshWeather },
                        Action::App
                    );
                }
            }
            Message::UpdateRefreshInterval(value) => {
                self.refresh_input = value.clone();
                if let Ok(interval) = value.parse::<u64>() {
                    if interval >= 1 && interval <= 1440 {
                        self.config.refresh_interval_minutes = interval;
                        self.save_config();
                    }
                }
            }
            Message::ToggleAutoLocation => {
                self.config.use_auto_location = !self.config.use_auto_location;
                self.save_config();

                if self.config.use_auto_location {
                    return Task::perform(
                        async {
                            detect_location().await
                                .map_err(|e| e.to_string())
                        },
                        |result| Action::App(Message::LocationDetected(result)),
                    );
                }
            }
            Message::DetectLocation => {
                return Task::perform(
                    async {
                        detect_location().await
                            .map_err(|e| e.to_string())
                    },
                    |result| Action::App(Message::LocationDetected(result)),
                );
            }
            Message::LocationDetected(result) => {
                match result {
                    Ok((lat, lon, location_name)) => {
                        self.config.latitude = lat;
                        self.config.longitude = lon;
                        self.config.location_name = location_name;
                        self.save_config();
                        return Task::perform(
                            async { Message::RefreshWeather },
                            Action::App
                        );
                    }
                    Err(e) => {
                        eprintln!("Failed to detect location: {}", e);
                    }
                }
            }
            Message::ToggleHourly => {
                self.hourly_expanded = !self.hourly_expanded;
            }
            Message::ToggleForecast => {
                self.forecast_expanded = !self.forecast_expanded;
            }
            Message::ToggleSettings => {
                self.settings_expanded = !self.settings_expanded;
            }
        }
        Task::none()
    }

    fn style(&self) -> Option<cosmic::iced_runtime::Appearance> {
        Some(cosmic::applet::style())
    }
}

impl Tempest {
    fn save_config(&self) {
        if let Some(ref handler) = self.config_handler {
            if let Err(e) = self.config.write_entry(handler) {
                eprintln!("Failed to save config: {}", e);
            }
        }
    }
}
