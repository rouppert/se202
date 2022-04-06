#![no_std]
#![no_main]
#[rtic::app(device = pac, dispatchers = [USART2])]

mod app {
    use tp_led_matrix::{Image, Color, matrix::Matrix, image};
    use cortex_m_rt::entry;
    use panic_probe as _;
    use dwt_systick_monotonic::DwtSystick;
    use dwt_systick_monotonic::ExtU32;
    use defmt_rtt as _;
    use stm32l4xx_hal::{pac, prelude::*};   // Just to link it in the executable (it provides the vector table)
    use super::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMonotonic = DwtSystick<80_000_000>;
    type Instant = <MyMonotonic as rtic::Monotonic>::Instant;

    #[shared]
    struct Shared {}

    #[local]
    struct Local {
        matrix: Matrix,
        image: Image
    }

    #[idle(local = [])]
    fn idle(cx: idle::Context) -> ! {
        loop {}
    }

    #[task(local = [matrix, image, next_line: usize = 0])]
    fn display(cx: display::Context, at: Instant) {
        // Display line next_line (cx.local.next_line) of
        // the image (cx.local.image) on the matrix (cx.local.matrix).
        // All those are mutable references.
        cx.local.matrix.send_row(*cx.local.next_line, cx.local.image.row(*cx.local.next_line));
        // Increment next_line up to 7 and wraparound to 0
        *cx.local.next_line = (*cx.local.next_line+1)%8;
        display::spawn_at(at + 1.secs()/(8*60), at + 1.secs()/(8*60)).unwrap();
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("defmt correctly initialized");

        let mut cp = cx.core;
        let dp = cx.device;

        let mut mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, 80_000_000);


        // Initialize the clocks, hardware and matrix using your existing code
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

        display::spawn(mono.now()).unwrap();

        // Return the resources and the monotonic timer
        (Shared {}, Local { matrix, image }, init::Monotonics(mono))
    }
}
