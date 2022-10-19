use ctrlc;

static mut ERRORS: usize = 0;
static mut CHECKS: usize = 0;

fn main() {
    let size = 8 * 1024 * 1024; // 8 GB

    let mut buf = Vec::new();

    let start_time = std::time::SystemTime::now();

    print!("initializing...");
    for i in 0..size {
        buf.push(i);
    }
    println!("done!");

    ctrlc::set_handler(move || unsafe {
        println!(
            "A total of {} errors found after checking {} times since {:?}",
            ERRORS, CHECKS, start_time
        );
        std::process::exit(0);
    })
    .expect("error setting ctrl-c handler");

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        for i in 0..buf.len() {
            if buf[i] != i {
                println!("error: 0x{:X} was set to 0x{:X}, resetting", i, buf[i]);
                buf[i] = i;
                unsafe { ERRORS += 1 };
            }
        }

        unsafe {
            println!("check {}", CHECKS);
            CHECKS += 1
        };
    }
}
