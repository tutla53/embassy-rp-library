//! Servo PIO Task with state machine 0 and 1 

#![no_std]
#![no_main]

use {
    core::time::Duration,
    rp2040_servo_pio::ServoPioBuilder,
    embassy_time::Timer,
    embassy_executor::Spawner,
    embassy_rp::{
        bind_interrupts,
        config::Config,
        usb::{Driver, InterruptHandler as UsbInterruptHandler},
        peripherals::{USB, PIO0},
        pio::{Pio, InterruptHandler as PioInterruptHandler},
        pio_programs::pwm::{PioPwm, PioPwmProgram},
    },
    {defmt_rtt as _, panic_probe as _},
};

bind_interrupts!(pub struct Irqs {
    PIO0_IRQ_0 => PioInterruptHandler<PIO0>;
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

const REFRESH_INTERVAL: u64 = 20000;

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner){
    let p = embassy_rp::init(Config::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let Pio { mut common, sm0, .. } = Pio::new(p.PIO0, Irqs);
    let prg = PioPwmProgram::new(&mut common);

    let servo_motor_pwm = PioPwm::new(&mut common, sm0, p.PIN_10, &prg);

    let mut servo_motor = ServoPioBuilder::new(servo_motor_pwm)
        .set_period(Duration::from_micros(REFRESH_INTERVAL))
        .set_max_degree_rotation(180)
        .set_min_pulse_width(Duration::from_micros(1000))
        .set_max_pulse_width(Duration::from_micros(2000))
        .set_initial_position(90)
        .build();

    servo_motor.stop();
    Timer::after_secs(1).await;
    servo_motor.start();
    Timer::after_secs(5).await;
    
    let mut target: u64 = 0;

    loop {
        log::info!("Current Pos {} - Target {}", servo_motor.get_current_pos(), target);
        
        log::info!("Waiting the servo to sweep....");
        servo_motor.sweep(target, 100).await;
        log::info!("Servo Sweep is Complete");
        
        if target == 0 {target = 180;}
        else {target = 0};

        Timer::after_millis(1).await;
    }
}