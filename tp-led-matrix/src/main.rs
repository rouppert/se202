#![no_std]
#![no_main]
#[rtic::app(device = pac, dispatchers = [USART2])]

mod app {
    use stm32l4xx_hal::device::USART1;
    use tp_led_matrix::{Image, Color, matrix::Matrix, image};
    use cortex_m_rt::entry;
    use panic_probe as _;
    use dwt_systick_monotonic::DwtSystick;
    use dwt_systick_monotonic::ExtU32;
    use defmt_rtt as _;
    use stm32l4xx_hal::{pac, prelude::*};   // Just to link it in the executable (it provides the vector table)
    use stm32l4xx_hal::serial::{Config, Event, Rx, Serial};
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
        matrix: Matrix,
        usart1_rx: Rx<USART1>,
        next_image: Image
    }

    #[idle(local = [])]
    fn idle(cx: idle::Context) -> ! {
        let mut count: i32 = 0;
        loop {}
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

    #[task(binds = USART1,
        local = [usart1_rx, next_image, next_pos: usize = 0],
        shared = [image])]
    fn receive_byte(mut cx: receive_byte::Context)
    {
        let next_image: &mut Image = cx.local.next_image;
        let next_pos: &mut usize = cx.local.next_pos;
        if let Ok(b) = cx.local.usart1_rx.read() {
            // Handle the incoming byte according to the SE203 protocol
            // and update next_image
            // Do not forget that next_image.as_mut() might be handy here!
            if b == 0xff {*next_pos = 0;}
            else {
                next_image.as_mut()[*next_pos] =  b;
                *next_pos += 1;
            }
            // If the received image is complete, make it available to
            // the display task.
            if *next_pos == 8 * 8 * 3 {
                cx.shared.image.lock(|image| {
                    // Replace the image content by the new one, for example
                    // by swapping them, and reset next_pos
                    *image = *next_image;
                    *next_pos = 0;
                });
            }
        }
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
            
        let rx = gpiob.pb7.into_alternate::<7>(&mut gpiob.moder,&mut gpiob.otyper,&mut gpiob.afrl);
        let tx = gpiob.pb6.into_alternate::<7>(&mut gpiob.moder,&mut gpiob.otyper,&mut gpiob.afrl);
        let config = stm32l4xx_hal::serial::Config::default().baudrate(38400.bps());
        let mut serial = stm32l4xx_hal::serial::Serial::usart1(dp.USART1, (tx, rx), config, clocks, &mut rcc.apb2);
        serial.listen(Event::Rxne);
        let usart1_rx = serial.split().1;
        //*cx.next_image = Image::Default();
        
        let image = Image::default();
        let next_image = Image::default();
        display::spawn(mono.now()).unwrap();

        // Return the resources and the monotonic timer
        (Shared {image}, Local { matrix, usart1_rx, next_image }, init::Monotonics(mono))
    }
}
