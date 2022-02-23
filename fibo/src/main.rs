use clap::{Arg, App};



fn fibo(n: u32) -> Option<u32> {
    if n==0 {return Some(0)}
    if n==1 {return Some(1)}
    else {
        let mut ret_value: u32 = 0;
        match fibo(n-2) {
            None => return None,
            Some(v) => ret_value += v
        }

        match fibo(n-1) {
            None => return None,
            Some(v) => ret_value += v
        }
        return Some(ret_value); 
    }
}

fn main() {
    let matches = App::new("Fibo")
    .about("Compute Fibonacci suite values")
    //Les flags
    .arg(Arg::new("verbose")
        .short('v')
        .long("verbose")
        .help("Print intermediate value")
        .takes_value(false))
    .arg(Arg::new("version")
        .short('V')
        .long("version")
        .help("Print version information")
        .takes_value(false))
    //Les Inputs
    .arg(Arg::new("VALUE")
        .required(true)
        .help("The maximal number to print the fibo value of"))
    //Les options
    .arg(Arg::new("min")
        .short('m')
        .long("min")
        .help("The minimum number to compute")
        .takes_value(true)
        .value_name("NUMBER")).get_matches();


    for i in 0..51 {
        match fibo(i) {
            None => break,
            Some(v) => println!("{}", v)
        }
    }
}
