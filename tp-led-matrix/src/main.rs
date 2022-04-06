#![no_std]
#![no_main]
#[rtic::app(device = pac, dispatchers = [USART2, USART3])]

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
    struct Shared {
        image: Image
    }

    #[local]
    struct Local {
        matrix: Matrix
    }

    #[idle(local = [])]
    fn idle(cx: idle::Context) -> ! {
        let mut count: i32 = 0;
        loop {
            if count==10_000-1 {defmt::info!("iteration"); count = 0}
            else {count=count+1;}
        }
    }

    #[task(local = [matrix, next_row: usize = 0], shared = [image], priority = 2)]
    fn display(mut cx: display::Context, at: Instant) {
        // Display line next_line (cx.local.next_line) of
        // the image (cx.local.image) on the matrix (cx.local.matrix).
        // All those are mutable references.
        cx.shared.image.lock(|image| {
            // Here you can use image, which is a &mut Image,
            // to display the appropriate row
            cx.local.matrix.send_row(*cx.local.next_row, image.row(*cx.local.next_row));
            // Increment next_line up to 7 and wraparound to 0
            *cx.local.next_row = (*cx.local.next_row+1)%8;
        });
    
        
        display::spawn_at(at + 1.secs()/(8*60), at + 1.secs()/(8*60)).unwrap();
    }

    #[task(local = [], shared = [image], priority = 1)]
    fn rotate_image(mut cx: rotate_image::Context, color_index: usize) {
        cx.shared.image.lock(|image| {
            match(color_index) {
                0=>*image = Image::gradient(image::RED),
                1=>*image = Image::gradient(image::GREEN),
                2=>*image = Image::gradient(image::BLUE),
                _=>panic!()
            }
        });
        let next: usize = (color_index+1)%3;
        rotate_image::spawn_after(1.secs(), next).unwrap();
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

        let image = Image::default();
        rotate_image::spawn(0).unwrap();
        display::spawn(mono.now()).unwrap();

        // Return the resources and the monotonic timer
        (Shared {image}, Local { matrix }, init::Monotonics(mono))
    }
}
