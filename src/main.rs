#[macro_use]
extern crate clap; // clap is still not rust 2018 compatible

mod instruction;
mod machine;
mod screen;

fn main() {
    let mut app = clap::app_from_crate!()
        .arg(clap::Arg::with_name("file").required(true).help("ROM file"))
        .arg(
            clap::Arg::with_name("freq")
                .short("f")
                .default_value("10")
                .help("clock frequency (60hz * this) [max: 255]"),
        );

    let matches = app.clone().get_matches();

    let freq = matches.value_of("freq").unwrap().parse::<u8>();
    if let Err(_) = freq {
        println!("Error: invalid freq\n");
        app.print_help();
        println!();
        return;
    }
    let freq = freq.unwrap();

    let mut machine = machine::Machine::open(freq, matches.value_of("file").unwrap()).unwrap();

    let mut loop_helper = spin_sleep::LoopHelper::builder()
        .build_with_target_rate(60.0 * freq as f64);

    loop {
        loop_helper.loop_start();

        machine.step([false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false]);

        if machine.screen().needs_redraw() {
            print!("\x1B[2J");
            for y in 0..32 {
                for x in 0..64 {
                    if machine.screen().buffer()[y][x] {
                        print!("##");
                    } else {
                        print!("..");
                    }
                }
                println!();
            }
            machine.screen().redrawn();
        }

        loop_helper.loop_sleep();
    }
}