#[macro_use]
extern crate clap; // clap is still not rust 2018 compatible

mod frontends;
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
    if freq.is_err() {
        println!("Error: invalid freq\n");
        app.print_help().unwrap();
        println!();
        return;
    }
    let freq = freq.unwrap();

    let machine = machine::Machine::open(freq, matches.value_of("file").unwrap()).unwrap();
    let mut frontend = frontends::GlutinWindow::new();

    frontend.run(freq, machine);
}
