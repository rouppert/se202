#![no_std]
#![no_main]

use cortex_m_rt::entry;
use panic_probe as _;
use defmt_rtt as _;
use stm32l4 as _;   // Just to link it in the executable (it provides the vector table)

#[entry]
fn main() -> ! {
    defmt::info!("Hello, world!");
    panic!("The program stopped");
}
