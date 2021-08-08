use chrono::Local;

#[test]
fn test_date_fromat() {
    let date_str = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{}", date_str)
}
