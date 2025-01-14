//! Servo Task

#![no_std]
#![no_main]

use {
    rp2040_servo::ServoBuilder,
    embassy_executor::Spawner,
    embassy_rp::{
        bind_interrupts,
        config::Config,
        pwm::{Config as PwmConfig, Pwm},
        usb::{Driver, InterruptHandler as UsbInterruptHandler},
        peripherals::USB,
    },
    embassy_time::Timer,
};

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => UsbInterruptHandler<USB>;
});

const SERVO_FREQ: u32 = 50;

#[embassy_executor::task]
async fn logger_task(driver: Driver<'static, USB>) {
    embassy_usb_logger::run!(1024, log::LevelFilter::Info, driver);
}

#[embassy_executor::main]
async fn main(spawner: Spawner){
    let p = embassy_rp::init(Config::default());
    let driver = Driver::new(p.USB, Irqs);
    spawner.spawn(logger_task(driver)).unwrap();

    let servo_pwm_device = Pwm::new_output_a(p.PWM_SLICE5, p.PIN_10, PwmConfig::default());
    
    let mut servo_motor = ServoBuilder::new(servo_pwm_device)
        .set_servo_freq(SERVO_FREQ)
        .set_max_degree_rotation(180)
        .set_min_pulse_width(1000)
        .set_max_pulse_width(2000)
        .set_initial_position(0)
        .build();

    servo_motor.enable();
    Timer::after_secs(1).await;
    log::info!("Curren Pos: {}", servo_motor.get_current_pos());

    loop {
        servo_motor.rotate(180);
        log::info!("Curren Pos: {}", servo_motor.get_current_pos());
        Timer::after_secs(5).await;
     
        servo_motor.rotate(90);
        log::info!("Curren Pos: {}", servo_motor.get_current_pos());
        Timer::after_secs(5).await;
    }
}