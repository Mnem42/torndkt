pub fn to_hms(seconds: i64) -> String{
    format!("{:0>2}:{:0>2}:{:0>2}",
            (seconds / 3600),
            (seconds % 3600 / 60),
            (seconds % 60)
    )
}