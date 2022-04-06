#![no_std]
#![no_main]

use tp_led_matrix::{Image, Color, matrix::Matrix, image};
use cortex_m_rt::entry;
use panic_probe as _;
use defmt_rtt as _;
use stm32l4xx_hal::{pac, prelude::*};   // Just to link it in the executable (it provides the vector table)

#[entry]
fn main() -> ! {
    let cp = pac::CorePeripherals::take().unwrap();
    let dp = pac::Peripherals::take().unwrap();
    run(cp, dp)
}

fn run(_cp: pac::CorePeripherals, dp: pac::Peripherals) -> ! {
    // Get high-level representations of hardware modules
    let mut rcc = dp.RCC.constrain();
    let mut flash = dp.FLASH.constrain();
    let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

    // Setup the clocks at 80MHz using HSI (by default since HSE/MSI are not configured).
    // The flash wait states will be configured accordingly.
    let clocks = rcc.cfgr.sysclk(80.MHz()).freeze(&mut flash.acr, &mut pwr);
    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
    let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
    let mut matrix = Matrix::new(
        gpioa.pa2,
        gpioa.pa3,
        gpioa.pa4,
        gpioa.pa5,
        gpioa.pa6,
        gpioa.pa7,
        gpioa.pa15,
        gpiob.pb0,
        gpiob.pb1,
        gpiob.pb2,
        gpioc.pc3,
        gpioc.pc4,
        gpioc.pc5,
        &mut gpioa.moder,
        &mut gpioa.otyper,
        &mut gpiob.moder,
        &mut gpiob.otyper,
        &mut gpioc.moder,
        &mut gpioc.otyper,
        clocks);
    let image: Image=Image::gradient(image::BLUE);
    loop {
        matrix.display_image(&image);
    }
}
