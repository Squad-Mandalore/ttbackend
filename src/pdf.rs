use crate::database::set_up_database;
use crate::models::Worktime;
use crate::service::worktime;
use chrono::{DateTime, Datelike, Local, NaiveDate};
use printpdf::*;
use sqlx::postgres::types::PgInterval;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;

fn add_month(given_month: &str) -> String {
    let year: i32 = given_month[0..4].parse().unwrap();
    let month: u32 = given_month[5..7].parse().unwrap();

    let new_month = if month == 12 { 1 } else { month + 1 };

    let new_year = if month == 12 { year + 1 } else { year };
    format!("{:04}-{:02}", new_year, new_month)
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

fn generate_schedule(worktimes: Vec<Worktime>, year: i32, month: u32) -> Vec<Vec<String>> {
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
                let task_description = worktime.task_id;

                // Handle the duration
                if let Some(d) = &worktime.timeduration {
                    let formatted_duration = format_duration(d.clone());
                    day_entry.push(format!("{:?}, {}", worktime.work_type, task_description));
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
            eprintln!("Invalid date: {}-{}-{}", year, month, day);
        }
    }

    schedule
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

pub async fn get_month_times(given_month: &str, employee_id: i32) -> sqlx::Result<Vec<Vec<String>>> {
    let database_pool = set_up_database().await;
    let next_month = add_month(given_month);
    let year: i32 = given_month[0..4].parse().expect("Invalid year format");
    let month: u32 = given_month[5..7].parse().expect("Invalid month format");

    // Query for getting the worktime of the emplyee in given month
    let datetime_start_stc = format!("{}-01T00:00:00Z", given_month);
    let datetime_start = DateTime::parse_from_rfc3339(&datetime_start_stc).unwrap();

    let datetime_end_stc = format!("{}-01T00:00:00Z", next_month);
    let datetime_end = DateTime::parse_from_rfc3339(&datetime_end_stc).unwrap();

    let worktimes = worktime::get_timers_in_boundary(
        employee_id,
        datetime_start,
        datetime_end,
        &database_pool,
    )
    .await?;

    // generate schedule
    let schedule = generate_schedule(worktimes, year, month);

    Ok(schedule)
}

pub async fn generate_pdf(given_month: &str, employee_id: i32, first_name: &str, last_name: &str, email: &str) {
    let (doc, page1, layer1) =
        PdfDocument::new("Zeiterfassungen", Mm(210.0), Mm(297.0), "Layer 1");

    let font_bold = doc
        .add_external_font(File::open("fonts/ntn-Bold.ttf").unwrap())
        .unwrap();
    let font_medium = doc
        .add_external_font(File::open("fonts/ntn-Medium.ttf").unwrap())
        .unwrap();
    let font_light = doc
        .add_external_font(File::open("fonts/ntn-Light.ttf").unwrap())
        .unwrap();

    let mut current_layer = doc.get_page(page1).get_layer(layer1);

    let mut month_time_work = 0;
    let mut month_time_ride = 0;

    // Header Info's
    let points = vec![
        (Point::new(Mm(0.0), Mm(217.0)), false),
        (Point::new(Mm(0.0), Mm(298.0)), false),
        (Point::new(Mm(211.0), Mm(298.0)), false),
        (Point::new(Mm(211.0), Mm(217.0)), false),
    ];
    let rectangle = Line {
        points,
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    };

    let header_colour = Color::Rgb(Rgb::new(0.176, 0.176, 0.176, None));
    current_layer.set_fill_color(header_colour);
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
    current_layer.use_text(
        email,
        12.0,
        Mm(60.0),
        Mm(234.0),
        &font_light,
    );

    // Day on which the pdf was requested
    let current_date = Local::now().date_naive();
    let formatted_date = current_date.format("%d.%m.%Y").to_string();

    // Month of the requested 
    let given_date = NaiveDate::parse_from_str(&format!("{}-01", given_month), "%Y-%m-%d").unwrap();
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

    // Body-Info Info's
    let points = vec![
        (Point::new(Mm(21.0), Mm(150.0)), false),
        (Point::new(Mm(21.0), Mm(201.0)), false),
        (Point::new(Mm(188.0), Mm(201.0)), false),
        (Point::new(Mm(188.0), Mm(150.0)), false),
    ];
    let rectangle = Line {
        points,
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    };

    // Body Info Colour
    let header_colour = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
    current_layer.set_fill_color(header_colour);
    current_layer.add_shape(rectangle);

    // Body Info Text
    let text_colour = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    current_layer.set_fill_color(text_colour);

    let used_schedule = get_month_times(given_month, employee_id).await.unwrap();
    let mut days_in_month = 0;

    // Iterate Days of Month
    for (_day_num, day_entry) in used_schedule.iter().enumerate() {
        days_in_month += 1;
        // Combined total work and ride times for that day (for the header)
        let mut combined_work_time: u64 = 0;
        let mut combined_ride_time: u64 = 0;

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
    let month_time_work_average_text = format_minutes_as_time(month_time_work/days_in_month);
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
    let month_time_ride_average_text = format_minutes_as_time(month_time_ride/days_in_month);
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
    let month_time_combined_text = format_minutes_as_time(month_time_ride+month_time_work);
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
    let month_time_combined_average_text = format_minutes_as_time((month_time_work + month_time_ride)/days_in_month);
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

    // Table Header
    let points = vec![
        (Point::new(Mm(21.0), Mm(115.0)), false),
        (Point::new(Mm(21.0), Mm(127.0)), false),
        (Point::new(Mm(188.0), Mm(127.0)), false),
        (Point::new(Mm(188.0), Mm(115.0)), false),
    ];
    let rectangle = Line {
        points,
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    };
    let header_colour = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
    current_layer.set_fill_color(header_colour);
    current_layer.add_shape(rectangle);

    let text_colour = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    current_layer.set_fill_color(text_colour);

    let column_widths = [Mm(25.0), Mm(55.0), Mm(90.0), Mm(120.0), Mm(155.0)];

    current_layer.use_text("Datum", 11.0, column_widths[0], Mm(120.0), &font_bold);
    current_layer.use_text("Arbeitszeit", 11.0, column_widths[1], Mm(120.0), &font_bold);
    current_layer.use_text("Fahrzeit", 11.0, column_widths[2], Mm(120.0), &font_bold);
    current_layer.use_text("Gesamtzeit", 11.0, column_widths[3], Mm(120.0), &font_bold);
    current_layer.use_text("Aufgabe", 11.0, column_widths[4], Mm(120.0), &font_bold);

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

        let _col1 = format!("{}, {:02}.", day_entry[0], day_num + 1);
        let _col2 = "00:00".to_string();
        let _col3 = "00:00".to_string();
        let _col4 = "00:00".to_string();
        let _col5 = "Haus bauen".to_string();

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
            &*col5,
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

            let points = vec![
                (Point::new(Mm(21.0), current_y_pos - Mm(3.0)), false),
                (
                    Point::new(Mm(21.0), current_y_pos + line_height - Mm(3.0)),
                    false,
                ),
                (
                    Point::new(Mm(188.0), current_y_pos + line_height - Mm(3.0)),
                    false,
                ),
                (Point::new(Mm(188.0), current_y_pos - Mm(3.0)), false),
            ];
            let rectangle = Line {
                points,
                is_closed: true,
                has_fill: true,
                has_stroke: false,
                is_clipping_path: false,
            };
            let header_colour = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
            current_layer.set_fill_color(header_colour);
            current_layer.add_shape(rectangle);

            let text_colour = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
            current_layer.set_fill_color(text_colour);

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
                &*task_id,
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

    let pdf_name = format!("./target/Zeiterfassung {} {}.pdf", employee_id, given_month);
    doc.save(&mut BufWriter::new(
        File::create(pdf_name).unwrap(),
    ))
    .unwrap();
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

    let points = vec![
        (Point::new(Mm(21.0), Mm(273.0)), false),
        (Point::new(Mm(21.0), Mm(285.0)), false),
        (Point::new(Mm(188.0), Mm(285.0)), false),
        (Point::new(Mm(188.0), Mm(273.0)), false),
    ];
    let rectangle = Line {
        points,
        is_closed: true,
        has_fill: true,
        has_stroke: false,
        is_clipping_path: false,
    };
    let header_colour = Color::Rgb(Rgb::new(0.952, 0.952, 0.952, None));
    current_layer.set_fill_color(header_colour);
    current_layer.add_shape(rectangle);

    let text_colour = Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None));
    current_layer.set_fill_color(text_colour);

    current_layer.use_text("Datum", 11.0, column_widths[0], Mm(278.0), font_bold);
    current_layer.use_text("Arbeitszeit", 11.0, column_widths[1], Mm(278.0), font_bold);
    current_layer.use_text("Fahrzeit", 11.0, column_widths[2], Mm(278.0), font_bold);
    current_layer.use_text("Gesamtzeit", 11.0, column_widths[3], Mm(278.0), font_bold);
    current_layer.use_text("Aufgabe", 11.0, column_widths[4], Mm(278.0), font_bold);

    let new_y_pos = Mm(273.0) - line_height;

    // Return the new layer, updated y position, and incremented page number
    (current_layer, new_y_pos, current_page + 1)
}
