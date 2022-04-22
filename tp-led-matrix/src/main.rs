#![no_std]
#![no_main]
#[rtic::app(device = pac, dispatchers = [USART2])]

mod app {
    use stm32l4xx_hal::device::USART1;
    use tp_led_matrix::{Image, Color, matrix::Matrix, image};
    use cortex_m_rt::entry;
    use core::mem::MaybeUninit;
    use panic_probe as _;
    use heapless::pool::*;
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
        next_image: Option<Box<Image>>, //next image to be displayed
        pool: Pool<Image>
    }

    #[local]
    struct Local {
        matrix: Matrix,
        usart1_rx: Rx<USART1>,
        current_image: Box<Image>, //image to be displayed
        rx_image: Box<Image> //image sent by the user
    }

    #[idle(local = [])]
    //Simply waits
    fn idle(cx: idle::Context) -> ! {
        loop {}
    }

    #[task(local = [current_image, matrix, next_row: usize = 0], shared = [next_image, pool], priority = 2)]
    //Displays the current image
    fn display(mut cx: display::Context, at: Instant) {
        cx.local.matrix.send_row(*cx.local.next_row, cx.local.current_image.row(*cx.local.next_row));
       // Increment next_line up to 7 and wraparound to 0
        if *cx.local.next_row == 7 {
            (cx.shared.next_image, cx.shared.pool).lock(|next_image, pool| {
                if let Some(mut t) = next_image.take() {
                    core::mem::swap(&mut t, &mut *cx.local.current_image);
                    pool.free(t);
                }      
            });
        }
        *cx.local.next_row = (*cx.local.next_row+1)%8;
        display::spawn_at(at + 1.secs()/(8*60), at + 1.secs()/(8*60)).unwrap();
    }

    #[task(binds = USART1,
        local = [usart1_rx, rx_image, next_pos: usize = 0],
        shared = [next_image, pool])]
    //Adds the bytes sent by the users to next_image
    fn receive_byte(mut cx: receive_byte::Context)
    {
        if let Ok(b) = cx.local.usart1_rx.read() {
            // Handle the incoming byte according to the SE203 protocol
            // and update next_image
            // Do not forget that next_image.as_mut() might be handy here!
            if b == 0xff {*cx.local.next_pos = 0;}
            else {
                cx.local.rx_image.as_mut()[*cx.local.next_pos] =  b;
                *cx.local.next_pos += 1;
            }
            // If the received image is complete, make it available to
            // the display task.
            if *cx.local.next_pos == 8 * 8 * 3 {
                (cx.shared.next_image, cx.shared.pool).lock(|next_image, pool| {
                    if let Some(mut image) = next_image.take() {
                        pool.free(image);
                    }
                    let mut future_image = pool.alloc().unwrap().init(Image::default());
                    core::mem::swap(&mut future_image, &mut *cx.local.rx_image);
                    next_image.replace(future_image);     
                }); 
                *cx.local.next_pos = 0;
            }
        }
    }


    #[init]
    //Initializes the hardware and creates an empty image
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
        let pool: Pool<Image> = Pool::new();
        unsafe {
            static mut MEMORY: MaybeUninit<[Node<Image>; 3]> = MaybeUninit::uninit();
            pool.grow_exact(&mut MEMORY);   // static mut access is unsafe
        }
        let mut current_image = pool.alloc().unwrap().init(Image::default());
        let mut rx_image = pool.alloc().unwrap().init(Image::default());
        display::spawn(mono.now()).unwrap();

        // Return the resources and the monotonic timer
        (Shared {next_image: None, pool}, Local { matrix, usart1_rx, current_image, rx_image}, init::Monotonics(mono))
    }
}

