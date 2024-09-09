use crate::models::Worktime;
use crate::service::{task, worktime};
use anyhow::{anyhow, Context};
use base64::encode;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use printpdf::*;
use sqlx::postgres::types::PgInterval;
use sqlx::PgPool;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
use std::io::Cursor;

fn add_month(given_month: &str) -> anyhow::Result<String> {
    let year: i32 = given_month[0..4]
        .parse()
        .context("cannot extract year of given month")?;
    let month: u32 = given_month[5..7]
        .parse()
        .context("cannot extract month of given month")?;

    let new_month = if month == 12 { 1 } else { month + 1 };

    let new_year = if month == 12 { year + 1 } else { year };
    Ok(format!("{:04}-{:02}", new_year, new_month))
}

fn truncate_string(input: &str, max_length: usize) -> String {
    if input.chars().count() > max_length {
        let truncated: String = input.chars().take(max_length).collect();
        format!("{}...", truncated)
    } else {
        input.to_string()
    }
}

fn get_weekday_abbreviation(date: chrono::NaiveDate) -> String {
    match date.weekday() {
        chrono::Weekday::Mon => "Mo".to_string(),
        chrono::Weekday::Tue => "Di".to_string(),
        chrono::Weekday::Wed => "Mi".to_string(),
        chrono::Weekday::Thu => "Do".to_string(),
        chrono::Weekday::Fri => "Fr".to_string(),
        chrono::Weekday::Sat => "Sa".to_string(),
        chrono::Weekday::Sun => "So".to_string(),
    }
}

fn add_durations(d1: &PgInterval, d2: &PgInterval) -> PgInterval {
    PgInterval {
        months: d1.months + d2.months,
        days: d1.days + d2.days,
        microseconds: d1.microseconds + d2.microseconds,
    }
}

async fn generate_schedule(
    worktimes: Vec<Worktime>,
    year: i32,
    month: u32,
    database_pool: &PgPool,
) -> anyhow::Result<Vec<Vec<String>>> {
    let mut schedule: Vec<Vec<String>> = Vec::new();

    // Determine the number of days in the month
    let num_days = NaiveDate::from_ymd_opt(year, month, 1)
        .and_then(|date| date.with_month(month + 1))
        .map(|next_month| (next_month - chrono::Duration::days(1)).day())
        .unwrap_or(31);

    for day in 1..=num_days {
        if let Some(date) = NaiveDate::from_ymd_opt(year, month, day) {
            let mut day_entry = vec![get_weekday_abbreviation(date)];
            let mut total_duration = PgInterval {
                months: 0,
                days: 0,
                microseconds: 0,
            };

            // Filter worktimes for the current day
            for worktime in worktimes
                .iter()
                .filter(|w| w.start_time.date_naive() == date)
            {
                let task_description_raw =
                    match task::get_task_by_id(worktime.task_id, database_pool).await? {
                        Some(task) => task.task_description,
                        None => None,
                    };

                let task_description =
                    task_description_raw.unwrap_or("No description available".to_string());

                let truncated_task_description = truncate_string(&task_description, 14);

                // Handle the duration
                if let Some(d) = &worktime.timeduration {
                    let formatted_duration = format_duration(d.clone());
                    day_entry.push(format!(
                        "{:?}, {}",
                        worktime.work_type, truncated_task_description
                    ));
                    day_entry.push(formatted_duration);

                    // Add to total duration
                    total_duration = add_durations(&total_duration, d);
                }
            }

            // Add total duration as the last entry
            if total_duration.microseconds > 0 {
                let formatted_total = format_duration(total_duration);
                day_entry.push(format!("Total: {}", formatted_total));
            }

            // Add the day's entry to the schedule
            schedule.push(day_entry);
        } else {
            // Return an error if the date couldn't be generated
            return Err(anyhow!(
                "Failed to generate schedule for year {} and month {} at day {}.",
                year,
                month,
                day
            ));
        }
    }

    Ok(schedule)
}

fn format_duration(interval: PgInterval) -> String {
    let total_microseconds = interval.microseconds;

    // Convert microseconds to total seconds
    let total_seconds = total_microseconds / 1_000_000;

    // Calculate hours and minutes
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;

    // Return formatted string
    format!("{:02}:{:02}", hours, minutes)
}

async fn get_month_times(
    given_month: &str,
    employee_id: &i32,
    database_pool: &PgPool, // Pass the database_pool by reference
) -> anyhow::Result<Vec<Vec<String>>> {
    let next_month = add_month(given_month)?;
    let year: i32 = given_month[0..4].parse()?;
    let month: u32 = given_month[5..7].parse()?;

    // Construct datetime boundaries for the query
    let datetime_start_stc = format!("{}-01T00:00:00Z", given_month);
    let datetime_start = DateTime::parse_from_rfc3339(&datetime_start_stc)?;

    let datetime_end_stc = format!("{}-01T00:00:00Z", next_month);
    let datetime_end = DateTime::parse_from_rfc3339(&datetime_end_stc)?;

    // Query for worktimes within the given date range
    let worktimes =
        worktime::get_timers_in_boundary(employee_id, datetime_start, datetime_end, database_pool)
            .await?;

    // Generate schedule, handling any potential error
    let schedule = generate_schedule(worktimes, year, month, database_pool).await?;

    Ok(schedule)
}

pub async fn generate_pdf(
    given_month: &str,
    employee_id: &i32,
    database_pool: &PgPool,
    color_for_header: &str,
) -> anyhow::Result<String> {
    let pdf_height = 297.0;
    let pdf_width = 210.0;
    let zero = 0.0;
    let employee_info = sqlx::query!(
        "SELECT firstname, lastname, email FROM employee  WHERE employee_id = $1",
        employee_id
    )
    .fetch_one(database_pool)
    .await?;

    let first_name = employee_info.firstname.unwrap_or(String::from(""));
    let last_name = employee_info.lastname.unwrap_or(String::from(""));
    let email = employee_info.email;

    let (doc, page1, layer1) =
        PdfDocument::new("Zeiterfassungen", Mm(pdf_width), Mm(pdf_height), "Layer 1");

    let font_bold = doc
        .add_external_font(File::open("fonts/ntn-Bold.ttf").context("cannot load bold font")?)
        .context("cannot apply bold font")?;
    let font_medium = doc
        .add_external_font(File::open("fonts/ntn-Medium.ttf").context("cannot load medium font")?)
        .context("cannot apply medium font")?;
    let font_light = doc
        .add_external_font(File::open("fonts/ntn-Light.ttf").context("cannot load light font")?)
        .context("cannot apply light font")?;

    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let mut month_time_work = 0;
    let mut month_time_ride = 0;

    let pdf_header_height_start = 217.0;

    let telekom_funk = Rgb::new(0.0, 0.5, 0.0, None);
    let hardworking_brown = Rgb::new(0.447, 0.227, 0.067, None);
    let peasent_blue = Rgb::new(0.141, 0.486, 0.737, None);
    let grassy_fields = Rgb::new(0.345, 0.459, 0.016, None);
    let baumarkt_rot = Rgb::new(0.647, 0.051, 0.051, None);
    let schmidt_brand = Rgb::new(0.871, 0.102, 0.102, None);
    let default_grey = Rgb::new(0.176, 0.176, 0.176, None);

    let header_color = match color_for_header {
        "TelekomFunk" => Color::Rgb(telekom_funk),
        "HardworkingBrown" => Color::Rgb(hardworking_brown),
        "PeasentBlue" => Color::Rgb(peasent_blue),
        "GrassyFields" => Color::Rgb(grassy_fields),
        "BaumarktRot" => Color::Rgb(baumarkt_rot),
        "SchmidtBrand" => Color::Rgb(schmidt_brand),
        _ => Color::Rgb(default_grey),
    };

    let rectangle = create_rectangle(zero, pdf_width, pdf_height, pdf_header_height_start);
    current_layer.set_fill_color(header_color);
    current_layer.add_shape(rectangle);

    // Header Text
    let text_color = Color::Rgb(Rgb::new(1.0, 1.0, 1.0, None));
    current_layer.set_fill_color(text_color);
    current_layer.use_text("Zeitenübersicht", 24.0, Mm(21.0), Mm(271.0), &font_medium);

    // First Name
    current_layer.use_text("Vorname:", 12.0, Mm(21.0), Mm(244.0), &font_light);
    current_layer.use_text(first_name, 12.0, Mm(60.0), Mm(244.0), &font_light);

    // Last Name
    current_layer.use_text("Nachname:", 12.0, Mm(21.0), Mm(239.0), &font_light);
    current_layer.use_text(last_name, 12.0, Mm(60.0), Mm(239.0), &font_light);

    // Email
    current_layer.use_text("Email-Adresse:", 12.0, Mm(21.0), Mm(234.0), &font_light);
    current_layer.use_text(email, 12.0, Mm(60.0), Mm(234.0), &font_light);

    // Day on which the pdf was requested
    let current_date = Local::now().date_naive();
    let formatted_date = current_date.format("%d.%m.%Y").to_string();

    // Month of the requested
    let given_date = NaiveDate::parse_from_str(&format!("{}-01", given_month), "%Y-%m-%d")
        .context("given month has the wrong format")?;
    let given_date_year = given_date.year();
    let given_date_month = given_date.month();

    // German schedaddles
    let german_months = [
        "Januar",
        "Februar",
        "März",
        "April",
        "Mai",
        "Juni",
        "Juli",
        "August",
        "September",
        "Oktober",
        "November",
        "Dezember",
    ];
    let month_name = german_months[(given_date_month - 1) as usize];
    let formatted_given_month = format!("{} {}", month_name, given_date_year);

    // Month
    current_layer.use_text("Monat:", 12.0, Mm(148.0), Mm(239.0), &font_light);
    current_layer.use_text(
        formatted_given_month,
        12.0,
        Mm(168.0),
        Mm(239.0),
        &font_light,
    );

    // Date
    current_layer.use_text("Datum:", 12.0, Mm(148.0), Mm(234.0), &font_light);
    current_layer.use_text(formatted_date, 12.0, Mm(168.0), Mm(234.0), &font_light);

    // Schmidt's Handwerksbetrieb
    current_layer.use_text("Schmidt's", 12.0, Mm(170.0), Mm(275.0), &font_light);
    current_layer.use_text("Handwerksbetrieb", 12.0, Mm(152.0), Mm(270.0), &font_light);

    let pdf_body_x_left = 21.0;
    let pdf_body_x_right = 188.0;
    let pdf_body_y_top = 201.0;
    let pdf_body_y_bottom = 150.0;

    let rectangle = create_rectangle(
        pdf_body_x_left,
        pdf_body_x_right,
        pdf_body_y_top,
        pdf_body_y_bottom,
    );
    let header_color = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
    current_layer.set_fill_color(header_color);
    current_layer.add_shape(rectangle);

    // Body Info Text
    let text_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    current_layer.set_fill_color(text_color);

    let used_schedule = get_month_times(given_month, employee_id, database_pool).await?;
    let mut days_in_month = 0;

    // Iterate Days of Month
    for day_entry in used_schedule.iter() {
        days_in_month += 1;

        // Combined total work and ride times for that day (for the header)
        let mut combined_work_time: u64 = 0;
        let mut combined_ride_time: u64 = 0;

        // first entry of each entry is the abbreviation of the weekday, therefore skiped
        let mut iter = day_entry.iter().skip(1);
        let mut task_times: HashMap<String, (u64, u64)> = HashMap::new();

        // Entries of one Day
        while let Some(entry) = iter.next() {
            let task_id = entry.split(", ").nth(1).unwrap_or("").to_string();
            let time_str = iter
                .next()
                .map(|s| s.to_string())
                .unwrap_or("00:00".to_string());
            let time_minutes = parse_time_to_minutes(&time_str);

            if entry.starts_with("Work") {
                let task_entry = task_times.entry(task_id.clone()).or_insert((0, 0));
                task_entry.0 += time_minutes;
                combined_work_time += time_minutes;
                month_time_work += combined_work_time;
            } else if entry.starts_with("Ride") {
                let task_entry = task_times.entry(task_id.clone()).or_insert((0, 0));
                task_entry.1 += time_minutes;
                combined_ride_time += time_minutes;
                month_time_ride += combined_ride_time;
            }
        }
    }

    // Worktime
    current_layer.use_text(
        "Arbeitszeit diesen Monat:",
        10.0,
        Mm(29.0),
        Mm(192.0),
        &font_medium,
    );
    let month_time_work_text = format_minutes_as_time(month_time_work);
    let parts: Vec<&str> = month_time_work_text.split(':').collect();
    let formatted_text = format!("{}h {}m", parts[0], parts[1]);
    current_layer.use_text(&formatted_text, 10.0, Mm(130.0), Mm(192.0), &font_bold);

    current_layer.use_text(
        "durchschnittliche Zeit pro Tag:",
        10.0,
        Mm(29.0),
        Mm(187.0),
        &font_medium,
    );
    let month_time_work_average_text = format_minutes_as_time(month_time_work / days_in_month);
    let parts: Vec<&str> = month_time_work_average_text.split(':').collect();
    let formatted_text = format!("{}h {}m", parts[0], parts[1]);
    current_layer.use_text(formatted_text, 10.0, Mm(130.0), Mm(187.0), &font_bold);

    // Traveltime
    current_layer.use_text(
        "Fahrstunden diesen Monat:",
        10.0,
        Mm(29.0),
        Mm(177.0),
        &font_medium,
    );
    let month_time_ride_text = format_minutes_as_time(month_time_ride);
    let parts: Vec<&str> = month_time_ride_text.split(':').collect();
    let formatted_text = format!("{}h {}m", parts[0], parts[1]);
    current_layer.use_text(formatted_text, 10.0, Mm(130.0), Mm(177.0), &font_bold);

    current_layer.use_text(
        "durchschnittliche Zeit pro Tag:",
        10.0,
        Mm(29.0),
        Mm(172.0),
        &font_medium,
    );
    let month_time_ride_average_text = format_minutes_as_time(month_time_ride / days_in_month);
    let parts: Vec<&str> = month_time_ride_average_text.split(':').collect();
    let formatted_text = format!("{}h {}m", parts[0], parts[1]);
    current_layer.use_text(formatted_text, 10.0, Mm(130.0), Mm(172.0), &font_bold);

    // Timetime
    current_layer.use_text(
        "Gesamtzeit diesen Monat:",
        10.0,
        Mm(29.0),
        Mm(162.0),
        &font_medium,
    );
    let month_time_combined_text = format_minutes_as_time(month_time_ride + month_time_work);
    let parts: Vec<&str> = month_time_combined_text.split(':').collect();
    let formatted_text = format!("{}h {}m", parts[0], parts[1]);
    current_layer.use_text(formatted_text, 10.0, Mm(130.0), Mm(162.0), &font_bold);

    current_layer.use_text(
        "durchschnittliche Gesamtzeit pro Tag:",
        10.0,
        Mm(29.0),
        Mm(157.0),
        &font_medium,
    );
    let month_time_combined_average_text =
        format_minutes_as_time((month_time_work + month_time_ride) / days_in_month);
    let parts: Vec<&str> = month_time_combined_average_text.split(':').collect();
    let formatted_text = format!("{}h {}m", parts[0], parts[1]);
    current_layer.use_text(formatted_text, 10.0, Mm(130.0), Mm(157.0), &font_bold);

    current_layer.use_text(
        "Gesamtübersicht erfasster Zeiten:",
        14.0,
        Mm(21.0),
        Mm(135.0),
        &font_medium,
    );

    let pdf_table_header_x_left = 21.0;
    let pdf_table_header_x_right = 188.0;
    let pdf_table_header_y_top = 127.0;
    let pdf_table_header_y_bottom = 115.0;

    let rectangle = create_rectangle(
        pdf_table_header_x_left,
        pdf_table_header_x_right,
        pdf_table_header_y_top,
        pdf_table_header_y_bottom,
    );
    let header_color = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
    current_layer.set_fill_color(header_color);
    current_layer.add_shape(rectangle);

    let text_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    current_layer.set_fill_color(text_color);

    let column_widths = [Mm(25.0), Mm(55.0), Mm(90.0), Mm(120.0), Mm(155.0)];
    let column_heights = Mm(120.0);

    current_layer.use_text("Datum", 11.0, column_widths[0], column_heights, &font_bold);
    current_layer.use_text(
        "Arbeitszeit",
        11.0,
        column_widths[1],
        column_heights,
        &font_bold,
    );
    current_layer.use_text(
        "Fahrzeit",
        11.0,
        column_widths[2],
        column_heights,
        &font_bold,
    );
    current_layer.use_text(
        "Gesamtzeit",
        11.0,
        column_widths[3],
        column_heights,
        &font_bold,
    );
    current_layer.use_text(
        "Aufgabe",
        11.0,
        column_widths[4],
        column_heights,
        &font_bold,
    );

    let font_size = 10.0;
    let line_height = Mm(8.0);
    let mut current_y_pos = Mm(110.0) - Mm(3.0);
    let mut current_page = 1;

    // Iterate Days of Month
    for (day_num, day_entry) in used_schedule.iter().enumerate() {
        // Combined total work and ride times for that day (for the header)
        let mut combined_work_time: u64 = 0;
        let mut combined_ride_time: u64 = 0;
        let mut combined_total_time: u64 = 0;

        // New Page
        if current_y_pos <= Mm(25.0) {
            let current_page_text = current_page.to_string();
            current_layer.use_text(current_page_text, 13.0, Mm(185.0), Mm(14.0), &font_medium);

            let (new_layer, _new_y_pos, new_page) =
                add_new_page(&doc, current_page, &font_bold, &column_widths, line_height);

            current_layer = new_layer;
            current_y_pos = Mm(273.0) - line_height;
            current_page = new_page;
        }

        // first entry of each entry is the abbreviation of the weekday, therefore skiped
        let mut iter = day_entry.iter().skip(1);
        let mut task_times: HashMap<String, (u64, u64)> = HashMap::new();

        // Entries of one Day
        while let Some(entry) = iter.next() {
            let task_id = entry.split(", ").nth(1).unwrap_or("").to_string();
            let time_str = iter
                .next()
                .map(|s| s.to_string())
                .unwrap_or("00:00".to_string());
            let time_minutes = parse_time_to_minutes(&time_str);

            if entry.starts_with("Work") {
                let task_entry = task_times.entry(task_id.clone()).or_insert((0, 0));
                task_entry.0 += time_minutes; // Add to work_time
                combined_work_time += time_minutes;
                month_time_work += combined_work_time;
            } else if entry.starts_with("Ride") {
                let task_entry = task_times.entry(task_id.clone()).or_insert((0, 0));
                task_entry.1 += time_minutes; // Add to ride_time
                combined_ride_time += time_minutes;
                month_time_ride += combined_ride_time;
            } else if entry.starts_with("Total:") {
                combined_total_time += parse_time_to_minutes(&entry.replacen("Total: ", "", 1));
            }
        }

        // Display the header for the day
        let col1 = format!("{}, {:02}.", day_entry[0], day_num + 1);
        let col2 = format_minutes_as_time(combined_work_time);
        let col3 = format_minutes_as_time(combined_ride_time);
        let col4 = format_minutes_as_time(combined_total_time);
        let col5 = ""; // Empty for the header

        // Output header row for the day
        current_layer.use_text(
            &col1,
            font_size,
            column_widths[0],
            current_y_pos,
            &font_medium,
        );
        current_layer.use_text(
            &col2,
            font_size,
            column_widths[1],
            current_y_pos,
            &font_medium,
        );
        current_layer.use_text(
            &col3,
            font_size,
            column_widths[2],
            current_y_pos,
            &font_medium,
        );
        current_layer.use_text(
            &col4,
            font_size,
            column_widths[3],
            current_y_pos,
            &font_medium,
        );
        current_layer.use_text(
            col5,
            font_size,
            column_widths[4],
            current_y_pos,
            &font_medium,
        );

        // Move the position down for the next row
        current_y_pos -= line_height;

        // Output the task rows (combine "Work" and "Ride" times for the same task_id)
        for (task_id, (work_time, ride_time)) in task_times.iter() {
            // New Page
            if current_y_pos <= Mm(25.0) {
                let current_page_text = current_page.to_string();
                current_layer.use_text(current_page_text, 13.0, Mm(185.0), Mm(14.0), &font_medium);

                // Call the refactored function to add a new page
                let (new_layer, _new_y_pos, new_page) =
                    add_new_page(&doc, current_page, &font_bold, &column_widths, line_height);

                // Update current_layer, current_y_pos, and current_page with the returned values
                current_layer = new_layer;
                current_y_pos = Mm(273.0) - line_height;
                current_page = new_page;
            }

            let work_time_str = format_minutes_as_time(*work_time);
            let ride_time_str = format_minutes_as_time(*ride_time);

            // Calculate the total time (work + ride)
            let total_time = work_time + ride_time;
            let total_time_str = format_minutes_as_time(total_time);

            let pdf_day_entry_x_left = 21.0;
            let pdf_day_entry_x_right = 188.0;
            let pdf_day_entry_y_top = current_y_pos.0 + line_height.0 - 3.0;
            let pdf_day_entry_y_bottom = current_y_pos.0 - 3.0;

            let rectangle = create_rectangle(
                pdf_day_entry_x_left,
                pdf_day_entry_x_right,
                pdf_day_entry_y_top,
                pdf_day_entry_y_bottom,
            );
            let header_color = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
            current_layer.set_fill_color(header_color);
            current_layer.add_shape(rectangle);

            let text_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
            current_layer.set_fill_color(text_color);

            // Output task_id rows
            current_layer.use_text("", font_size, column_widths[0], current_y_pos, &font_medium); // Empty first column
            current_layer.use_text(
                &work_time_str,
                font_size,
                column_widths[1],
                current_y_pos,
                &font_medium,
            );
            current_layer.use_text(
                &ride_time_str,
                font_size,
                column_widths[2],
                current_y_pos,
                &font_medium,
            );
            current_layer.use_text(
                &total_time_str,
                font_size,
                column_widths[3],
                current_y_pos,
                &font_medium,
            );
            current_layer.use_text(
                task_id,
                font_size,
                column_widths[4],
                current_y_pos,
                &font_medium,
            );

            current_y_pos -= line_height;
        }
    }
    let current_page_text = current_page.to_string();
    current_layer.use_text(current_page_text, 13.0, Mm(185.0), Mm(14.0), &font_medium);

    // Write the PDF to memory (Vec<u8> buffer)
    let mut pdf_buffer = Vec::new();

    {
        let cursor = Cursor::new(&mut pdf_buffer);
        let mut writer = BufWriter::new(cursor);
        doc.save(&mut writer)?;
    }
    let pdf_base64 = encode(&pdf_buffer);

    Ok(pdf_base64)
}

// Helper function to parse time string (e.g., "08:00") to minutes
fn parse_time_to_minutes(time_str: &str) -> u64 {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 2 {
        return 0;
    }
    let hours: u64 = parts[0].parse().unwrap_or(0);
    let minutes: u64 = parts[1].parse().unwrap_or(0);
    hours * 60 + minutes
}

// Helper function to format minutes as time (e.g., 480 minutes -> "08:00")
fn format_minutes_as_time(minutes: u64) -> String {
    let hours = minutes / 60;
    let minutes = minutes % 60;
    format!("{:02}:{:02}", hours, minutes)
}

fn add_new_page(
    doc: &PdfDocumentReference,
    current_page: i32,
    font_bold: &IndirectFontRef,
    column_widths: &[Mm],
    line_height: Mm,
) -> (PdfLayerReference, Mm, i32) {
    let (new_page, new_layer) = doc.add_page(
        Mm(210.0),
        Mm(297.0),
        &format!("Page {}, Layer 1", current_page),
    );
    let current_layer = doc.get_page(new_page).get_layer(new_layer);

    let pdf_table_header_x_left = 21.0;
    let pdf_table_header_x_right = 188.0;
    let pdf_table_header_y_top = 285.0;
    let pdf_table_header_y_bottom = 273.0;

    let rectangle = create_rectangle(
        pdf_table_header_x_left,
        pdf_table_header_x_right,
        pdf_table_header_y_top,
        pdf_table_header_y_bottom,
    );
    let header_color = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
    current_layer.set_fill_color(header_color);
    current_layer.add_shape(rectangle);

    let text_color = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    current_layer.set_fill_color(text_color);

    current_layer.use_text("Datum", 11.0, column_widths[0], Mm(278.0), font_bold);
    current_layer.use_text("Arbeitszeit", 11.0, column_widths[1], Mm(278.0), font_bold);
    current_layer.use_text("Fahrzeit", 11.0, column_widths[2], Mm(278.0), font_bold);
    current_layer.use_text("Gesamtzeit", 11.0, column_widths[3], Mm(278.0), font_bold);
    current_layer.use_text("Aufgabe", 11.0, column_widths[4], Mm(278.0), font_bold);

    let new_y_pos = Mm(273.0) - line_height;

    // Return the new layer, updated y position, and incremented page number
    (current_layer, new_y_pos, current_page + 1)
}

// Clean Code function to create an rectange
fn create_rectangle(x_left: f64, x_right: f64, y_top: f64, y_bottom: f64) -> Line {
    let points = vec![
        (Point::new(Mm(x_left), Mm(y_bottom)), false),
        (Point::new(Mm(x_left), Mm(y_top)), false),
        (Point::new(Mm(x_right), Mm(y_top)), false),
        (Point::new(Mm(x_right), Mm(y_bottom)), false),
    ];

    Line {
        points,
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    }
}

// Tests module
#[cfg(test)]
mod tests {
    use base64::decode;
    use std::error::Error;
    use std::fs::{self};
    use std::io::Write;
    use std::path::Path;
    use tokio::fs as tokio_fs;

    use super::*;

    // Helper function for writing the pdf file
    async fn write_b64_to_file(path: &str, base64_data: &str) -> Result<(), Box<dyn Error>> {
        tokio_fs::write(path, base64_data).await?;
        Ok(())
    }

    fn save_as_pdf(b64_path: &str, pdf_path: &str) -> Result<(), Box<dyn Error>> {
        // Read the Base64-encoded content from the .b64 file
        let b64_content = fs::read_to_string(b64_path)?;

        // Decode the Base64 string into bytes
        let pdf_bytes = decode(b64_content)?;

        // Write the decoded bytes to a PDF file
        let mut pdf_file = fs::File::create(pdf_path)?;
        pdf_file.write_all(&pdf_bytes)?;

        Ok(())
    }

    #[test]
    fn test_format_minutes_as_time() {
        assert_eq!(format_minutes_as_time(60), "01:00");
        assert_eq!(format_minutes_as_time(0), "00:00");
        assert_eq!(format_minutes_as_time(1), "00:01");
        assert_eq!(format_minutes_as_time(10), "00:10");
        assert_eq!(format_minutes_as_time(61), "01:01");
        assert_eq!(format_minutes_as_time(6000), "100:00");
    }

    #[sqlx::test(fixtures(
        "../fixtures/truncate.sql",
        "../fixtures/task.sql",
        "../fixtures/address.sql",
        "../fixtures/employee.sql",
        "../fixtures/worktime.sql"
    ))]
    fn test_generate_pdf(pool: sqlx::PgPool) -> Result<(), Box<dyn Error>> {
        let generated_pdf = generate_pdf("2024-01", &1, &pool, "a").await?;

        let output_path = "test/generated_output.b64";
        println!("{}", output_path);
        write_b64_to_file(output_path, &generated_pdf).await?;

        assert!(
            Path::new(output_path).exists(),
            "The .b64 file was not created!"
        );

        let pdf_output_path = "test/output.pdf";
        save_as_pdf(output_path, pdf_output_path)?;

        fs::remove_file(output_path)?;
        Ok(())
    }

    #[test]
    fn test_parse_time_to_minutes() {
        assert_eq!(parse_time_to_minutes("01:00"), 60);
        assert_eq!(parse_time_to_minutes("00:00"), 0);
        assert_eq!(parse_time_to_minutes("00:01"), 1);
        assert_eq!(parse_time_to_minutes("00:10"), 10);
        assert_eq!(parse_time_to_minutes("01:01"), 61);
        assert_eq!(parse_time_to_minutes("100:00"), 6000);
    }

    #[test]
    fn test_add_month() {
        assert_eq!(add_month("2024-01").unwrap(), "2024-02");
        assert_eq!(add_month("2024-12").unwrap(), "2025-01");
        assert_eq!(add_month("2023-11").unwrap(), "2023-12");
    }

    #[test]
    fn test_get_weekday_abbreviation() {
        assert_eq!(
            get_weekday_abbreviation(NaiveDate::from_ymd_opt(2024, 9, 2).unwrap()),
            "Mo"
        );
        assert_eq!(
            get_weekday_abbreviation(NaiveDate::from_ymd_opt(2024, 9, 3).unwrap()),
            "Di"
        );
        assert_eq!(
            get_weekday_abbreviation(NaiveDate::from_ymd_opt(2024, 9, 4).unwrap()),
            "Mi"
        );
        assert_eq!(
            get_weekday_abbreviation(NaiveDate::from_ymd_opt(2024, 9, 5).unwrap()),
            "Do"
        );
        assert_eq!(
            get_weekday_abbreviation(NaiveDate::from_ymd_opt(2024, 9, 6).unwrap()),
            "Fr"
        );
        assert_eq!(
            get_weekday_abbreviation(NaiveDate::from_ymd_opt(2024, 9, 7).unwrap()),
            "Sa"
        );
        assert_eq!(
            get_weekday_abbreviation(NaiveDate::from_ymd_opt(2024, 9, 8).unwrap()),
            "So"
        );
    }

    #[test]
    fn test_add_durations() {
        let d1 = PgInterval {
            months: 2,
            days: 10,
            microseconds: 5_000_000,
        };
        let d2 = PgInterval {
            months: 3,
            days: 5,
            microseconds: 7_000_000,
        };

        let result = add_durations(&d1, &d2);

        assert_eq!(result.months, 5);
        assert_eq!(result.days, 15);
        assert_eq!(result.microseconds, 12_000_000);
    }

    #[test]
    fn test_format_duration() {
        let duration = PgInterval {
            months: 0,
            days: 0,
            microseconds: 3660 * 1_000_000, // 1 hour, 1 minute
        };

        assert_eq!(format_duration(duration), "01:01");

        let duration_zero = PgInterval {
            months: 0,
            days: 0,
            microseconds: 0,
        };

        assert_eq!(format_duration(duration_zero), "00:00");
    }

    #[test]
    fn test_truncate_string() {
        let input = "Short text";
        let result = truncate_string(input, 20);
        assert_eq!(result, "Short text");

        let input = "Exactly twenty char";
        let result = truncate_string(input, 20);
        assert_eq!(result, "Exactly twenty char");

        let input = "This is a very long text that should be truncated";
        let result = truncate_string(input, 20);
        assert_eq!(result, "This is a very long ...");

        let input = "Text";
        let result = truncate_string(input, 0);
        assert_eq!(result, "...");

        let input = "";
        let result = truncate_string(input, 10);
        assert_eq!(result, "");
    }
}
