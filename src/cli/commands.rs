use inquire::Confirm;

use chrono::{Datelike, TimeZone, Utc};

use crate::bluetooth::scanner;
use crate::devices::manager::DeviceManager;
use crate::devices::models::Device;
use crate::error::ScanError;
use crate::protocol::bigdata::{sleep_phase_label, SleepData};
use crate::protocol::hr::HeartRateResult;
use crate::protocol::steps::StepsResult;
use crate::tui;

pub async fn scan(filter_colmi: bool) {
    match filter_devices(filter_colmi).await {
        Ok(devices) => {
            println!("Found {} device(s):", devices.len());
            for (i, device) in devices.iter().enumerate() {
                println!("  {}. {}", i + 1, device.display_name());
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn connect(filter_colmi: bool) {
    match filter_devices(filter_colmi).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::connect_and_setup(&selected_device).await {
                    Ok(_) => println!("Connected and configured device: {}", selected_device),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn battery() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::get_battery_level(&selected_device).await {
                    Ok(response) => println!("{}", response),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn blink() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::blink(&selected_device).await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn hr(days: u32) {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                for day_offset in 0..days {
                    let day = Utc::now() - chrono::Duration::days(day_offset as i64);
                    let midnight = Utc
                        .with_ymd_and_hms(day.year(), day.month(), day.day(), 0, 0, 0)
                        .single()
                        .unwrap_or(day);

                    match DeviceManager::get_heart_rate_log(
                        &selected_device,
                        midnight.timestamp() as u32,
                    )
                    .await
                    {
                        Ok(HeartRateResult::Log(log)) => {
                            let readings: Vec<u8> = log
                                .heart_rates
                                .iter()
                                .copied()
                                .filter(|&r| r > 0)
                                .collect();
                            if readings.is_empty() {
                                println!("{}: no readings", midnight.format("%Y-%m-%d"));
                            } else {
                                let avg = readings.iter().map(|&r| r as u32).sum::<u32>()
                                    / readings.len() as u32;
                                let min = readings.iter().min().unwrap();
                                let max = readings.iter().max().unwrap();
                                println!(
                                    "{}: {} readings, avg {} bpm ({} - {}), interval {}m",
                                    midnight.format("%Y-%m-%d"),
                                    readings.len(),
                                    avg,
                                    min,
                                    max,
                                    log.range
                                );
                            }
                        }
                        Ok(HeartRateResult::NoData) => {
                            println!("{}: no data", midnight.format("%Y-%m-%d"));
                        }
                        Err(err) => {
                            println!("{}", err);
                        }
                    }
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn steps(days: u32) {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                for day_offset in 0..days {
                    match DeviceManager::get_steps(&selected_device, day_offset as i8).await {
                        Ok(StepsResult::Details(details)) => {
                            if details.is_empty() {
                                println!("Day -{}: no activity", day_offset);
                                continue;
                            }
                            let total_steps: u32 = details.iter().map(|d| d.steps as u32).sum();
                            let total_calories: u32 = details.iter().map(|d| d.calories).sum();
                            let total_distance: u32 =
                                details.iter().map(|d| d.distance as u32).sum();
                            let first = &details[0];
                            let date = format!(
                                "{:04}-{:02}-{:02}",
                                first.year, first.month, first.day
                            );
                            let first_slot = details.iter().map(|d| d.time_index).min().unwrap();
                            let last_slot = details.iter().map(|d| d.time_index).max().unwrap();
                            let fmt_slot = |slot: u8| format!("{:02}:{:02}", slot / 4, (slot % 4) * 15);
                            println!(
                                "Day -{} ({}): {} steps, {} kcal, {} m, active {}–{}",
                                day_offset,
                                date,
                                total_steps,
                                total_calories,
                                total_distance,
                                fmt_slot(first_slot),
                                fmt_slot(last_slot)
                            );
                        }
                        Ok(StepsResult::NoData) => println!("Day -{}: no data", day_offset),
                        Err(err) => println!("{}", err),
                    }
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn sleep() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::get_sleep(&selected_device).await {
                    Ok(SleepData { days }) => {
                        if days.is_empty() {
                            println!("No sleep data available");
                        } else {
                            for day in &days {
                                let total: u16 = day.phases.iter().map(|p| p.minutes as u16).sum();
                                println!(
                                    "Sleep {} nights ago: {}h {:02}m ({}:{:02} → {}:{:02})",
                                    day.days_ago,
                                    total / 60,
                                    total % 60,
                                    day.start_minutes / 60,
                                    day.start_minutes % 60,
                                    day.end_minutes / 60,
                                    day.end_minutes % 60
                                );
                                let mut breakdown: Vec<(&str, u16)> = Vec::new();
                                for phase in &day.phases {
                                    let label = sleep_phase_label(phase.phase_type);
                                    if let Some(entry) =
                                        breakdown.iter_mut().find(|(l, _)| *l == label)
                                    {
                                        entry.1 += phase.minutes as u16;
                                    } else {
                                        breakdown.push((label, phase.minutes as u16));
                                    }
                                }
                                for (label, minutes) in breakdown {
                                    println!("  {}: {}m", label, minutes);
                                }
                            }
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn reset() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match Confirm::new("This will reset the device. Continue?")
                    .with_default(false)
                    .prompt()
                {
                    Ok(true) => match DeviceManager::reset(&selected_device).await {
                        Ok(_) => (),
                        Err(err) => {
                            println!("{}", err);
                        }
                    },
                    Ok(false) => {
                        println!("Reset cancelled.");
                    }
                    Err(err) => {
                        println!("{}", err);
                    }
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn reboot() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::reboot(&selected_device).await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

pub async fn find() {
    match filter_devices(true).await {
        Ok(devices) => {
            println!("Found {} device(s):", &devices.len());

            if let Some(selected_device) = tui::select_device(devices) {
                match DeviceManager::find(&selected_device).await {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                    }
                };
            }
        }
        Err(err) => println!("{}", err),
    }
}

async fn filter_devices(filter_colmi: bool) -> Result<Vec<Device>, ScanError> {
    let devices = scanner::scan_for_devices().await?;

    let filtered_devices = if filter_colmi {
        devices
            .into_iter()
            .filter(|d| d.is_colmi_device())
            .collect::<Vec<Device>>()
    } else {
        devices
    };

    if filtered_devices.is_empty() {
        return Err(if filter_colmi {
            ScanError::NoColmiDevices
        } else {
            ScanError::NoDevices
        });
    }

    Ok(filtered_devices)
}
