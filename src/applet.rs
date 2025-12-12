// SPDX-License-Identifier: GPL-3.0-only

use cosmic::app::{Core, Task};
use cosmic::cosmic_config::CosmicConfigEntry;
use cosmic::iced::platform_specific::shell::wayland::commands::popup::{destroy_popup, get_popup};
use cosmic::iced::window::Id;
use cosmic::iced::{Limits, Subscription};
use cosmic::iced_futures::Subscription as IcedSubscription;
use cosmic::widget::{self, settings, text};
use cosmic::{Action, Application, Element};
use std::collections::HashSet;
use std::time::Duration;

use crate::config::{Config, MeasurementSystem, PopupTab, TemperatureUnit};
use crate::weather::{
    aqi_standard_label, aqi_to_description, detect_location, fetch_air_quality, fetch_alerts,
    fetch_weather, format_date, format_hour, format_time, is_night_time, search_city,
    uses_imperial_units, weathercode_to_description, weathercode_to_icon_name,
    wind_direction_to_compass, AirQualityData, Alert, AlertSeverity, AqiStandard, LocationResult,
    WeatherData,
};

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// This is the struct that represents your application.
/// It is used to define the data that will be used by your application.
pub struct Tempest {
    /// Application state which is managed by the COSMIC runtime.
    core: Core,
    /// The popup id.
    popup: Option<Id>,
    /// Weather data.
    weather_data: Option<WeatherData>,
    /// Air quality data.
    air_quality: Option<AirQualityData>,
    /// Active weather alerts.
    alerts: Vec<Alert>,
    /// IDs of alerts already shown as notifications (prevents duplicates).
    seen_alert_ids: HashSet<String>,
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
    /// Current AQI for panel display
    current_aqi: Option<(i32, AqiStandard)>,
    /// Loading state
    is_loading: bool,
    /// Error state
    error_message: Option<String>,
    /// Active tab in the popup
    active_tab: PopupTab,
    /// Cached formatted timestamp for display (avoids recomputing on every render)
    last_updated_display: Option<String>,
}

impl Default for Tempest {
    fn default() -> Self {
        let config = Config::default();
        Self {
            core: Default::default(),
            popup: None,
            weather_data: None,
            air_quality: None,
            alerts: Vec::new(),
            seen_alert_ids: HashSet::new(),
            city_input: String::new(),
            refresh_input: config.refresh_interval_minutes.to_string(),
            search_results: Vec::new(),
            display_label: "...".to_string(),
            current_weathercode: 0,
            current_aqi: None,
            is_loading: true,
            error_message: None,
            active_tab: PopupTab::default(),
            last_updated_display: None,
            config,
            config_handler: None,
        }
    }
}

/// Message variants for application communication.
#[derive(Debug, Clone)]
pub enum Message {
    TogglePopup,
    PopupClosed(Id),
    RefreshWeather,
    WeatherUpdated(Result<WeatherData, String>),
    AirQualityUpdated(Result<AirQualityData, String>),
    AlertsUpdated(Result<Vec<Alert>, String>),
    Tick,
    ToggleTemperatureUnit,
    ToggleAlertsEnabled,
    ToggleShowAqiInPanel,
    ToggleAutoUnits,
    UpdateCityInput(String),
    SearchCity,
    CitySearchResult(Result<Vec<LocationResult>, String>),
    SelectLocation(usize),
    UpdateRefreshInterval(String),
    DetectLocation,
    LocationDetected(Result<(f64, f64, String, String), String>),
    ToggleAutoLocation,
    SelectTab(PopupTab),
    OpenUrl(String),
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
        let active_tab = config.default_tab;

        let app = Tempest {
            core,
            config: config.clone(),
            config_handler,
            city_input: String::new(),
            refresh_input,
            search_results: Vec::new(),
            display_label: "...".to_string(),
            active_tab,
            ..Default::default()
        };

        // Start with auto-location if enabled, otherwise fetch weather
        let task = if config.use_auto_location {
            Task::perform(
                async { detect_location().await.map_err(|e| e.to_string()) },
                |result| Action::App(Message::LocationDetected(result)),
            )
        } else {
            Task::perform(async { Message::RefreshWeather }, Action::App)
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
            },
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
        use chrono::{Local, Timelike};
        use cosmic::iced::Alignment;

        // Determine if it's night time using actual sunrise/sunset data
        let is_night = self
            .weather_data
            .as_ref()
            .and_then(|w| w.forecast.first())
            .map(|day| is_night_time(&day.sunrise, &day.sunset))
            .unwrap_or_else(|| {
                // Fallback to 6pm-6am if no weather data available
                let hour = Local::now().hour();
                !(6..18).contains(&hour)
            });

        // Use error icon if there's an error, otherwise use weather icon
        let icon_name = if self.error_message.is_some() {
            "dialog-error-symbolic"
        } else {
            weathercode_to_icon_name(self.current_weathercode, is_night)
        };

        let icon = widget::icon::from_name(icon_name).size(16).symbolic(true);

        let temperature_text = text(&self.display_label);

        let has_alerts = !self.alerts.is_empty();
        let alert_icon = widget::icon::from_name("dialog-warning-symbolic")
            .size(18)
            .symbolic(true);

        let data = if self.core.applet.is_horizontal() {
            let mut row = widget::row()
                .align_y(Alignment::Center)
                .spacing(4);
            if has_alerts {
                row = row.push(alert_icon);
            }
            row = row.push(icon).push(temperature_text);
            if self.config.show_aqi_in_panel {
                if let Some((aqi, _)) = self.current_aqi {
                    row = row.push(text("|").size(12));
                    row = row.push(text(format!("AQI {}", aqi)));
                }
            }
            Element::from(row)
        } else {
            let mut col = widget::column()
                .align_x(Alignment::Center)
                .spacing(4);
            if has_alerts {
                col = col.push(alert_icon);
            }
            col = col.push(icon).push(temperature_text);
            if self.config.show_aqi_in_panel {
                if let Some((aqi, _)) = self.current_aqi {
                    col = col.push(text(format!("AQI {}", aqi)).size(12));
                }
            }
            Element::from(col)
        };

        let button = widget::button::custom(data)
            .class(cosmic::theme::Button::AppletIcon)
            .on_press(Message::TogglePopup);

        widget::autosize::autosize(button, widget::Id::unique()).into()
    }

    fn view_window(&self, _id: Id) -> Element<'_, Self::Message> {
        let mut column = widget::column()
            .spacing(10)
            .padding(10)
            .width(cosmic::iced::Length::Fixed(420.0));

        // Header row with timestamp and action buttons
        let has_alerts = !self.alerts.is_empty();
        let alerts_icon = if has_alerts {
            "dialog-warning-symbolic"
        } else {
            "weather-clear-symbolic"
        };

        let mut header = widget::row()
            .spacing(8)
            .align_y(cosmic::iced::Alignment::Center);

        // Add timestamp if available
        if let Some(ref formatted_time) = self.last_updated_display {
            header = header.push(text(format!("Updated: {}", formatted_time)).size(12));
        }

        // Alert button - styled to stand out when alerts are active
        let alerts_btn = widget::button::icon(widget::icon::from_name(alerts_icon))
            .on_press(Message::SelectTab(PopupTab::Alerts))
            .padding(6);
        let alerts_btn = if has_alerts {
            alerts_btn.class(cosmic::theme::Button::Destructive)
        } else {
            alerts_btn
        };

        header = header
            .push(widget::horizontal_space())
            .push(
                widget::button::icon(widget::icon::from_name("view-refresh-symbolic"))
                    .on_press(Message::RefreshWeather)
                    .padding(6),
            )
            .push(alerts_btn)
            .push(
                widget::button::icon(widget::icon::from_name("emblem-system-symbolic"))
                    .on_press(Message::SelectTab(PopupTab::Settings))
                    .padding(6),
            );

        column = column.push(header);

        // Prominent location display
        column = column.push(
            widget::container(text(&self.config.location_name).size(18))
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill),
        );

        column = column.push(widget::divider::horizontal::default());

        // Show error message if there is one
        if let Some(ref error) = self.error_message {
            column = column.push(
                widget::container(
                    widget::column()
                        .spacing(10)
                        .push(widget::icon::from_name("dialog-error-symbolic").size(48))
                        .push(text("Failed to load weather").size(18))
                        .push(text(error).size(14))
                        .push(widget::button::standard("Retry").on_press(Message::RefreshWeather)),
                )
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill),
            );
        } else if self.is_loading {
            column = column.push(
                widget::container(
                    widget::column()
                        .spacing(10)
                        .align_x(cosmic::iced::alignment::Horizontal::Center)
                        .push(widget::icon::from_name("content-loading-symbolic").size(48))
                        .push(text("Loading weather data...").size(18)),
                )
                .align_x(cosmic::iced::alignment::Horizontal::Center)
                .width(cosmic::iced::Length::Fill),
            );
        } else if let Some(ref weather) = self.weather_data {
            // Tab bar - 4 tabs only (Alerts/Settings accessible via header buttons)
            let tab_bar = widget::row()
                .spacing(8)
                .align_y(cosmic::iced::Alignment::Center)
                .push(self.tab_button("Current", PopupTab::Current))
                .push(self.tab_button("Hourly", PopupTab::Hourly))
                .push(self.tab_button("7-Day", PopupTab::Forecast))
                .push(self.tab_button("Air", PopupTab::AirQuality));

            // Tab bar
            column = column.push(
                widget::container(tab_bar)
                    .align_x(cosmic::iced::alignment::Horizontal::Center)
                    .width(cosmic::iced::Length::Fill),
            );
            column = column.push(widget::divider::horizontal::default());

            // Tab content
            match self.active_tab {
                PopupTab::Current => {
                    // Temperature and condition
                    column = column.push(
                        widget::row()
                            .spacing(10)
                            .push(
                                text(self.config.temperature_unit.format(weather.current.temperature))
                                    .size(32),
                            )
                            .push(text(weathercode_to_description(
                                weather.current.weathercode,
                            ))),
                    );

                    // Feels like and humidity
                    column = column.push(
                        widget::row()
                            .spacing(20)
                            .push(
                                text(format!(
                                    "Feels like: {:.0}{}",
                                    weather.current.feels_like,
                                    self.config.temperature_unit.symbol()
                                ))
                                .size(14),
                            )
                            .push(
                                text(format!("Humidity: {}%", weather.current.humidity)).size(14),
                            ),
                    );

                    // Wind information
                    let wind_unit = self.config.measurement_system.wind_speed_unit();
                    column = column.push(
                        widget::row()
                            .spacing(20)
                            .push(
                                text(format!(
                                    "Wind: {:.1} {} {}",
                                    weather.current.windspeed,
                                    wind_unit,
                                    wind_direction_to_compass(weather.current.wind_direction)
                                ))
                                .size(14),
                            )
                            .push(
                                text(format!(
                                    "Gusts: {:.1} {}",
                                    weather.current.wind_gusts, wind_unit
                                ))
                                .size(14),
                            ),
                    );

                    // UV and cloud cover
                    column = column.push(
                        widget::row()
                            .spacing(20)
                            .push(
                                text(format!("UV Index: {:.1}", weather.current.uv_index)).size(14),
                            )
                            .push(
                                text(format!("Cloud Cover: {}%", weather.current.cloud_cover))
                                    .size(14),
                            ),
                    );

                    // Visibility and pressure
                    let visibility = self
                        .config
                        .measurement_system
                        .convert_visibility(weather.current.visibility);
                    let visibility_unit = self.config.measurement_system.visibility_unit();
                    column = column.push(
                        widget::row()
                            .spacing(20)
                            .push(
                                text(format!("Visibility: {:.1} {}", visibility, visibility_unit))
                                    .size(14),
                            )
                            .push(
                                text(format!("Pressure: {:.0} hPa", weather.current.pressure))
                                    .size(14),
                            ),
                    );

                    // Sunrise/Sunset
                    if let Some(first_day) = weather.forecast.first() {
                        column = column.push(
                            widget::row()
                                .spacing(20)
                                .push(
                                    text(format!("Sunrise: {}", format_time(&first_day.sunrise)))
                                        .size(14),
                                )
                                .push(
                                    text(format!("Sunset: {}", format_time(&first_day.sunset)))
                                        .size(14),
                                ),
                        );
                    }
                }
                PopupTab::AirQuality => {
                    if let Some(ref aq) = self.air_quality {
                        let label = aqi_standard_label(aq.standard);
                        let description = aqi_to_description(aq.aqi, aq.standard);

                        column = column.push(
                            widget::row()
                                .spacing(20)
                                .push(text(format!("{}: {}", label, aq.aqi)).size(16))
                                .push(text(description).size(14)),
                        );

                        column = column.push(
                            widget::row()
                                .spacing(20)
                                .push(text(format!("PM2.5: {:.1} ug/m3", aq.pm2_5)).size(14))
                                .push(text(format!("PM10: {:.1} ug/m3", aq.pm10)).size(14)),
                        );

                        column = column.push(
                            widget::row()
                                .spacing(20)
                                .push(text(format!("Ozone: {:.1} ug/m3", aq.ozone)).size(14))
                                .push(
                                    text(format!("NO2: {:.1} ug/m3", aq.nitrogen_dioxide)).size(14),
                                ),
                        );

                        column =
                            column.push(text(format!("CO: {:.1} ug/m3", aq.carbon_monoxide)).size(14));
                    } else {
                        column = column.push(text("Air quality data unavailable").size(14));
                    }
                }
                PopupTab::Alerts => {
                    if !self.config.alerts_enabled {
                        column = column.push(
                            widget::container(
                                widget::column()
                                    .spacing(10)
                                    .align_x(cosmic::iced::alignment::Horizontal::Center)
                                    .push(text("Weather alerts are disabled").size(14))
                                    .push(text("Enable them in Settings").size(12)),
                            )
                            .align_x(cosmic::iced::alignment::Horizontal::Center)
                            .width(cosmic::iced::Length::Fill),
                        );
                    } else if self.alerts.is_empty() {
                        column = column.push(
                            widget::container(
                                widget::column()
                                    .spacing(10)
                                    .align_x(cosmic::iced::alignment::Horizontal::Center)
                                    .push(
                                        widget::icon::from_name("weather-clear-symbolic")
                                            .size(48)
                                            .symbolic(true),
                                    )
                                    .push(text("No active alerts").size(16))
                                    .push(text("Your area is clear").size(12)),
                            )
                            .align_x(cosmic::iced::alignment::Horizontal::Center)
                            .width(cosmic::iced::Length::Fill),
                        );
                    } else {
                        for alert in &self.alerts {
                            let severity_icon = match alert.severity {
                                AlertSeverity::Extreme => "dialog-error-symbolic",
                                AlertSeverity::Severe => "dialog-warning-symbolic",
                                AlertSeverity::Moderate => "dialog-information-symbolic",
                                _ => "weather-severe-alert-symbolic",
                            };

                            column = column.push(
                                widget::container(
                                    widget::column()
                                        .spacing(4)
                                        .push(
                                            widget::row()
                                                .spacing(8)
                                                .push(
                                                    widget::icon::from_name(severity_icon)
                                                        .size(20)
                                                        .symbolic(true),
                                                )
                                                .push(text(&alert.event).size(14)),
                                        )
                                        .push(text(&alert.headline).size(12))
                                        .push_maybe(if alert.description.is_empty() {
                                            None
                                        } else {
                                            Some(
                                                widget::container(
                                                    widget::scrollable(
                                                        text(&alert.description).size(11),
                                                    )
                                                    .height(cosmic::iced::Length::Fixed(100.0)),
                                                )
                                                .padding([4, 0, 4, 0]),
                                            )
                                        })
                                        .push(
                                            text(format!(
                                                "Expires: {}",
                                                alert.expires.format("%b %d %I:%M %p")
                                            ))
                                            .size(10),
                                        ),
                                )
                                .padding(8)
                                .width(cosmic::iced::Length::Fill),
                            );
                            column = column.push(widget::divider::horizontal::default());
                        }
                    }
                }
                PopupTab::Hourly => {
                    // 4-column grid layout for hourly forecast
                    let hours_per_row = 4;
                    for chunk in weather.hourly.chunks(hours_per_row) {
                        let mut row = widget::row().spacing(8);

                        for hour in chunk {
                            let cell = widget::column()
                                .spacing(4)
                                .align_x(cosmic::iced::alignment::Horizontal::Center)
                                .push(text(format_hour(&hour.time)).size(12))
                                .push(
                                    widget::icon::from_name(weathercode_to_icon_name(
                                        hour.weathercode,
                                        false,
                                    ))
                                    .size(20)
                                    .symbolic(true),
                                )
                                .push(
                                    text(self.config.temperature_unit.format(hour.temperature))
                                        .size(14),
                                )
                                .push(
                                    text(format!("{}%", hour.precipitation_probability)).size(11),
                                );

                            row = row.push(
                                widget::container(cell)
                                    .width(cosmic::iced::Length::FillPortion(1))
                                    .align_x(cosmic::iced::alignment::Horizontal::Center),
                            );
                        }

                        // Pad incomplete rows with empty space
                        for _ in chunk.len()..hours_per_row {
                            row = row.push(
                                widget::container(widget::Space::new(0, 0))
                                    .width(cosmic::iced::Length::FillPortion(1)),
                            );
                        }

                        column = column.push(row);
                    }
                }
                PopupTab::Forecast => {
                    // Table header
                    column = column.push(
                        widget::row()
                            .spacing(8)
                            .push(
                                text("Day")
                                    .size(12)
                                    .width(cosmic::iced::Length::Fixed(80.0)),
                            )
                            .push(widget::Space::new(24, 0))
                            .push(
                                text("High")
                                    .size(12)
                                    .width(cosmic::iced::Length::Fixed(45.0)),
                            )
                            .push(
                                text("Low")
                                    .size(12)
                                    .width(cosmic::iced::Length::Fixed(45.0)),
                            )
                            .push(text("Conditions").size(12)),
                    );
                    column = column.push(widget::divider::horizontal::default());

                    // Data rows
                    for day in &weather.forecast {
                        column = column.push(
                            widget::row()
                                .spacing(8)
                                .align_y(cosmic::iced::Alignment::Center)
                                .push(
                                    text(format_date(&day.date))
                                        .size(13)
                                        .width(cosmic::iced::Length::Fixed(80.0)),
                                )
                                .push(
                                    widget::icon::from_name(weathercode_to_icon_name(
                                        day.weathercode,
                                        false,
                                    ))
                                    .size(20)
                                    .symbolic(true),
                                )
                                .push(
                                    text(self.config.temperature_unit.format(day.temp_max))
                                        .size(13)
                                        .width(cosmic::iced::Length::Fixed(45.0)),
                                )
                                .push(
                                    text(self.config.temperature_unit.format(day.temp_min))
                                        .size(13)
                                        .width(cosmic::iced::Length::Fixed(45.0)),
                                )
                                .push(text(weathercode_to_description(day.weathercode)).size(12)),
                        );
                    }
                }
                PopupTab::Settings => {
                    // Units section
                    column = column.push(settings::item(
                        "Temperature Unit",
                        widget::button::standard(self.config.temperature_unit.as_str())
                            .on_press(Message::ToggleTemperatureUnit),
                    ));

                    column = column.push(settings::item(
                        "Auto-select Units",
                        widget::row()
                            .spacing(8)
                            .align_y(cosmic::iced::Alignment::Center)
                            .push(
                                widget::toggler(self.config.auto_units)
                                    .on_toggle(|_| Message::ToggleAutoUnits),
                            )
                            .push(text("Based on location").size(11)),
                    ));

                    column = column.push(widget::divider::horizontal::default());

                    // Location section
                    column = column.push(settings::item(
                        "Auto-detect Location",
                        widget::toggler(self.config.use_auto_location)
                            .on_toggle(|_| Message::ToggleAutoLocation),
                    ));

                    if self.config.use_auto_location {
                        column = column.push(settings::item(
                            "",
                            widget::button::standard("Detect Now")
                                .on_press(Message::DetectLocation),
                        ));
                    }

                    column = column.push(settings::item(
                        "Current Location",
                        text(&self.config.location_name).size(13),
                    ));

                    if !self.config.use_auto_location {
                        column = column.push(settings::item(
                            "Search Location",
                            widget::row()
                                .spacing(8)
                                .push(
                                    widget::text_input("Enter city name...", &self.city_input)
                                        .on_input(Message::UpdateCityInput)
                                        .on_submit(|_| Message::SearchCity)
                                        .width(cosmic::iced::Length::Fixed(180.0)),
                                )
                                .push(
                                    widget::button::standard("Search")
                                        .on_press(Message::SearchCity),
                                ),
                        ));

                        if !self.search_results.is_empty() {
                            for (idx, result) in self.search_results.iter().enumerate() {
                                column = column.push(
                                    widget::button::text(&result.display_name)
                                        .on_press(Message::SelectLocation(idx))
                                        .padding(8)
                                        .width(cosmic::iced::Length::Fill),
                                );
                            }
                        }
                    }

                    column = column.push(widget::divider::horizontal::default());

                    // Refresh & Alerts section
                    column = column.push(settings::item(
                        "Refresh Interval",
                        widget::row()
                            .spacing(8)
                            .align_y(cosmic::iced::Alignment::Center)
                            .push(
                                widget::text_input("15", &self.refresh_input)
                                    .on_input(Message::UpdateRefreshInterval)
                                    .width(cosmic::iced::Length::Fixed(60.0)),
                            )
                            .push(text("minutes").size(13)),
                    ));

                    column = column.push(settings::item(
                        "Weather Alerts",
                        widget::row()
                            .spacing(8)
                            .align_y(cosmic::iced::Alignment::Center)
                            .push(
                                widget::toggler(self.config.alerts_enabled)
                                    .on_toggle(|_| Message::ToggleAlertsEnabled),
                            )
                            .push(text("US & EU").size(11)),
                    ));

                    column = column.push(settings::item(
                        "Show AQI in Panel",
                        widget::toggler(self.config.show_aqi_in_panel)
                            .on_toggle(|_| Message::ToggleShowAqiInPanel),
                    ));

                    column = column.push(widget::divider::horizontal::default());

                    // About section
                    column = column.push(settings::item(
                        "Version",
                        text(VERSION).size(13),
                    ));

                    column = column.push(settings::item(
                        "Support",
                        widget::button::text("Tip me on Ko-fi").on_press(Message::OpenUrl(
                            "https://ko-fi.com/vintagetechie".to_string(),
                        )),
                    ));
                }
            }

        }

        let scrollable = widget::scrollable(column).height(cosmic::iced::Length::Fill);

        self.core
            .applet
            .popup_container(scrollable)
            .limits(Self::popup_limits())
            .into()
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
                    popup_settings.positioner.size_limits = Self::popup_limits();
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
                let wind_unit = self
                    .config
                    .measurement_system
                    .wind_speed_api_param()
                    .to_string();
                let alerts_enabled = self.config.alerts_enabled;

                // Fetch weather and air quality in parallel
                let weather_task = Task::perform(
                    async move {
                        fetch_weather(lat, lon, &temp_unit, &wind_unit)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    |result| Action::App(Message::WeatherUpdated(result)),
                );

                let air_quality_task = Task::perform(
                    async move { fetch_air_quality(lat, lon).await.map_err(|e| e.to_string()) },
                    |result| Action::App(Message::AirQualityUpdated(result)),
                );

                // Fetch alerts if enabled
                let alerts_task = if alerts_enabled {
                    Task::perform(
                        async move { fetch_alerts(lat, lon).await.map_err(|e| e.to_string()) },
                        |result| Action::App(Message::AlertsUpdated(result)),
                    )
                } else {
                    Task::none()
                };

                return Task::batch([weather_task, air_quality_task, alerts_task]);
            }
            Message::WeatherUpdated(result) => {
                self.is_loading = false;

                match result {
                    Ok(data) => {
                        self.current_weathercode = data.current.weathercode;
                        self.display_label =
                            self.config.temperature_unit.format(data.current.temperature);
                        self.weather_data = Some(data);
                        self.error_message = None;

                        // Update last updated timestamp and cache formatted display
                        let now = chrono::Local::now();
                        self.config.last_updated = Some(now.timestamp());
                        self.last_updated_display = Some(
                            now.format("%I:%M %p")
                                .to_string()
                                .trim_start_matches('0')
                                .to_string(),
                        );
                        self.save_config();
                    }
                    Err(e) => {
                        eprintln!("Failed to fetch weather: {}", e);
                        self.display_label = "ERR".to_string();
                        self.current_weathercode = 0;
                        self.error_message = Some(e);
                    }
                }
            }
            Message::AirQualityUpdated(result) => match result {
                Ok(data) => {
                    self.current_aqi = Some((data.aqi, data.standard));
                    self.air_quality = Some(data);
                }
                Err(e) => {
                    eprintln!("Failed to fetch air quality: {}", e);
                    self.current_aqi = None;
                    self.air_quality = None;
                }
            },
            Message::AlertsUpdated(result) => match result {
                Ok(new_alerts) => {
                    // Send notifications for new alerts
                    for alert in &new_alerts {
                        if !self.seen_alert_ids.contains(&alert.id) {
                            self.send_alert_notification(alert);
                            self.seen_alert_ids.insert(alert.id.clone());
                        }
                    }
                    self.alerts = new_alerts;
                }
                Err(e) => {
                    eprintln!("Failed to fetch alerts: {}", e);
                }
            },
            Message::Tick => {
                return Task::perform(async { Message::RefreshWeather }, Action::App);
            }
            Message::ToggleTemperatureUnit => {
                // Toggle temperature unit and sync measurement system
                match self.config.temperature_unit {
                    TemperatureUnit::Fahrenheit => {
                        self.config.temperature_unit = TemperatureUnit::Celsius;
                        self.config.measurement_system = MeasurementSystem::Metric;
                    }
                    TemperatureUnit::Celsius => {
                        self.config.temperature_unit = TemperatureUnit::Fahrenheit;
                        self.config.measurement_system = MeasurementSystem::Imperial;
                    }
                };
                // Manual unit change disables auto-units
                self.config.auto_units = false;
                self.save_config();
                return Task::perform(async { Message::RefreshWeather }, Action::App);
            }
            Message::ToggleAlertsEnabled => {
                self.config.alerts_enabled = !self.config.alerts_enabled;
                if !self.config.alerts_enabled {
                    self.alerts.clear();
                }
                self.save_config();
                return Task::perform(async { Message::RefreshWeather }, Action::App);
            }
            Message::ToggleShowAqiInPanel => {
                self.config.show_aqi_in_panel = !self.config.show_aqi_in_panel;
                self.save_config();
            }
            Message::ToggleAutoUnits => {
                self.config.auto_units = !self.config.auto_units;
                self.save_config();
            }
            Message::UpdateCityInput(value) => {
                self.city_input = value;
            }
            Message::SearchCity => {
                let city = self.city_input.clone();
                if !city.is_empty() {
                    return Task::perform(
                        async move { search_city(&city).await.map_err(|e| e.to_string()) },
                        |result| Action::App(Message::CitySearchResult(result)),
                    );
                }
            }
            Message::CitySearchResult(result) => match result {
                Ok(results) => {
                    self.search_results = results;
                }
                Err(e) => {
                    eprintln!("City search failed: {}", e);
                    self.search_results.clear();
                }
            },
            Message::SelectLocation(idx) => {
                if let Some(location) = self.search_results.get(idx) {
                    let country = location.country.clone();
                    self.config.latitude = location.latitude;
                    self.config.longitude = location.longitude;
                    self.config.location_name = location.display_name.clone();
                    self.config.use_auto_location = false;
                    // Update manual location storage
                    self.config.manual_latitude = Some(location.latitude);
                    self.config.manual_longitude = Some(location.longitude);
                    self.config.manual_location_name = Some(location.display_name.clone());

                    self.apply_units_for_country(&country);

                    self.city_input.clear();
                    self.search_results.clear();
                    self.save_config();
                    return Task::perform(async { Message::RefreshWeather }, Action::App);
                }
            }
            Message::UpdateRefreshInterval(value) => {
                self.refresh_input = value.clone();
                if let Ok(interval) = value.parse::<u64>() {
                    if (1..=1440).contains(&interval) {
                        self.config.refresh_interval_minutes = interval;
                        self.save_config();
                    }
                }
            }
            Message::ToggleAutoLocation => {
                self.config.use_auto_location = !self.config.use_auto_location;

                if self.config.use_auto_location {
                    // Save current manual location before switching to auto
                    self.config.manual_latitude = Some(self.config.latitude);
                    self.config.manual_longitude = Some(self.config.longitude);
                    self.config.manual_location_name = Some(self.config.location_name.clone());
                    self.save_config();

                    return Task::perform(
                        async { detect_location().await.map_err(|e| e.to_string()) },
                        |result| Action::App(Message::LocationDetected(result)),
                    );
                } else {
                    // Restore previous manual location if available
                    if let (Some(lat), Some(lon), Some(name)) = (
                        self.config.manual_latitude,
                        self.config.manual_longitude,
                        self.config.manual_location_name.clone(),
                    ) {
                        self.config.latitude = lat;
                        self.config.longitude = lon;
                        self.config.location_name = name;
                    }
                    self.save_config();

                    return Task::perform(async { Message::RefreshWeather }, Action::App);
                }
            }
            Message::DetectLocation => {
                return Task::perform(
                    async { detect_location().await.map_err(|e| e.to_string()) },
                    |result| Action::App(Message::LocationDetected(result)),
                );
            }
            Message::LocationDetected(result) => match result {
                Ok((lat, lon, location_name, country)) => {
                    self.config.latitude = lat;
                    self.config.longitude = lon;
                    self.config.location_name = location_name;

                    self.apply_units_for_country(&country);

                    self.save_config();
                    return Task::perform(async { Message::RefreshWeather }, Action::App);
                }
                Err(e) => {
                    eprintln!("Failed to detect location: {}", e);
                }
            },
            Message::SelectTab(tab) => {
                self.active_tab = tab;
                self.config.default_tab = tab;
                self.save_config();
            }
            Message::OpenUrl(url) => {
                if let Err(e) = open::that(&url) {
                    eprintln!("Failed to open URL {}: {}", url, e);
                }
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

    /// Sends a desktop notification for a weather alert.
    fn send_alert_notification(&self, alert: &Alert) {
        use notify_rust::{Notification, Urgency};

        let urgency = match alert.severity {
            AlertSeverity::Extreme | AlertSeverity::Severe => Urgency::Critical,
            AlertSeverity::Moderate => Urgency::Normal,
            _ => Urgency::Low,
        };

        if let Err(e) = Notification::new()
            .summary(&alert.event)
            .body(&alert.headline)
            .icon("weather-severe-alert")
            .urgency(urgency)
            .show()
        {
            eprintln!("Failed to send alert notification: {}", e);
        }
    }

    /// Creates a tab button, highlighted if it matches the active tab.
    fn tab_button(&self, label: &'static str, tab: PopupTab) -> Element<'_, Message> {
        let btn = widget::button::text(label).on_press(Message::SelectTab(tab));
        if self.active_tab == tab {
            btn.class(cosmic::theme::Button::Suggested).into()
        } else {
            btn.into()
        }
    }

    /// Returns the size limits for the popup window.
    fn popup_limits() -> Limits {
        Limits::NONE
            .min_width(440.0)
            .max_width(440.0)
            .min_height(180.0)
            .max_height(550.0)
    }

    /// Sets temperature and measurement units based on country if auto_units is enabled.
    fn apply_units_for_country(&mut self, country: &str) {
        if self.config.auto_units {
            if uses_imperial_units(country) {
                self.config.temperature_unit = TemperatureUnit::Fahrenheit;
                self.config.measurement_system = MeasurementSystem::Imperial;
            } else {
                self.config.temperature_unit = TemperatureUnit::Celsius;
                self.config.measurement_system = MeasurementSystem::Metric;
            }
        }
    }
}
