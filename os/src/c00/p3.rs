use std::io::Error;

fn main() -> Result<(), Error> {
    use std::{thread, time};

    let five_minutes = time::Duration::from_secs(5);
    let now = time::Instant::now();
    thread::sleep(five_minutes);
    assert!(now.elapsed() >= five_minutes);

    let tar = "This is the target string";
    println!("{}", tar);

    use std::fs::File;
    use std::io::Write;

    let path = "p3.txt";
    let mut o_file = File::create(path)?;
    write!(o_file, "{}", tar)?;
    Ok(())
}