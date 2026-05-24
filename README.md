# SmartBin – Smart Recycling Bin with Rewards

An embedded project built on **STM32** in **Rust** (Embassy framework) that detects hand proximity, automatically opens the lid, and rewards the user through RFID scanning.

## Features

- **Hand detection** – HC-SR04 ultrasonic sensor measures distance in real time
- **Automatic lid** – SG90 servo opens the lid when a hand is within 30 cm
- **Audio feedback** – KY-006 buzzer beeps on each opening
- **Reward system** – RC522 RFID card adds 200 points per recycling action
- **Persistent memory** – points are saved to STM32 internal Flash and survive reboots
- **Temperature/humidity compensation** – speed of sound is calculated dynamically (`v = 331.4 + 0.606·T + 0.0124·H`) for more accurate distance readings

## Hardware

| Component | STM32 Pin |
|-----------|-----------|
| HC-SR04 TRIG | PC7 |
| HC-SR04 ECHO | PC8 |
| Servo SG90 | PB5 (TIM3 CH2) |
| Buzzer KY-006 | PB10 (TIM2 CH3) |
| RC522 SCK | PA5 |
| RC522 MOSI | PA7 |
| RC522 MISO | PA6 |
| RC522 CS | PA8 |
| RC522 RST | PA3 |

## Software

- **Language:** Rust (`no_std`, `no_main`)
- **Framework:** [Embassy](https://embassy.dev/) – async embedded
- **Libraries:** `embassy-stm32`, `embassy-time`, `mfrc522`, `defmt`

## How to run

```bash
cargo build
cargo run
```

> Requires `probe-rs` installed and an STM32 board connected via a debug probe.
