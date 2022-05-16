use serde::*;

// Dependencies for UART and I2C
use rust_i2c::{Command, Connection as I2c};
use rust_uart::{Connection as Uart};
use std::cell::RefCell;
use std::time::Duration;
use std::thread;
use serial::*;
// use rust_spi::{Connection as Spi};

use super::*;
use crate::InputEnum::*;

const I2C_GET: u8 = 0x01;
const I2C_SET: u8 = 0x10;
const UART_GET: u8 = 0x02;
const UART_SET: u8 = 0x20;

#[derive(Serialize,Deserialize)]
pub enum InputEnum{
    None,
    GetValues(ExampleEnum),
    SetValues(ExampleInput,ExampleEnum),
    GetI2c,
    SetI2c(u8),
    GetUart,
    SetUart(u8),
    // GetSpi(),
    // SetSpi(),
}

// Example of Struct containing the functions to connect to the payload
// #[derive(Serialize,Deserialize)]
pub struct ExampleStruct {
    // I2C connection
    i2c_connection: I2c,
    
    // UART connection
    uart_connection: Uart,
    // Buffer needed for UART connections
    buffer: RefCell<Vec<u8>>,

    // SPI connection 
    // for later use
    // spi_connection: Spi,

    last_error: ExampleError,
    last_command: InputEnum,
    // last_input: InputEnum,

    // example-values
    ex_no0: u16,
    ex_no1: u16,
    ex_str: String,
    ex_bool0: bool,
    ex_bool1: bool,
}
impl ExampleStruct {
    // basic function to initialise an instance of the ExampleStruct
    pub fn new(
        i2c_path: String,
        i2c_addr: u16,
        uart_path: String,
        uart_setting: serial::PortSettings,
        uart_timeout: Duration,
    ) -> ExampleResult<Self> {
        Ok(Self{
            i2c_connection: I2c::from_path(&i2c_path,i2c_addr),
            uart_connection: Uart::from_path(&uart_path,uart_setting,uart_timeout)?,
            buffer: RefCell::new(Vec::new()),
            // spi_connection: Spi::from_path(spi),

            last_error: ExampleError::None,
            last_command: InputEnum::None,
            
            ex_no0: 0u16,
            ex_no1: 0u16,
            ex_str: "".to_string(),
            ex_bool0: false,
            ex_bool1: false,
        })
    }

    // examples of get and set functions that use the previously defined
    // Enum and Structs as In-/Output
    pub fn get_values(&mut self, g: ExampleEnum) -> ExampleResult<ExampleOutput> {
        self.last_command = GetValues(g);
        match g {
            ExampleEnum::Zero => Ok(ExampleOutput{
                out_no: vec![self.ex_no0],
                out_str: self.ex_str.to_string(),
                out_bool: vec![self.ex_bool0],
            }),
            ExampleEnum::One => Ok(ExampleOutput{
                out_no: vec![self.ex_no1],
                out_str: self.ex_str.to_string(),
                out_bool: vec![self.ex_bool1],
            }),
            ExampleEnum::All => self.get_all_values()
        }
    }

    fn get_all_values(&self) -> ExampleResult<ExampleOutput> {
        Ok(ExampleOutput{
            out_no: vec![self.ex_no0,self.ex_no1],
            out_str: self.ex_str.to_string(),
            out_bool: vec![self.ex_bool0,self.ex_bool1],
        })
    }

    pub fn set_values(&mut self, s: ExampleInput, e: ExampleEnum) -> ExampleResult<()> {
        match e {
            ExampleEnum::Zero => {
                self.ex_no0 = s.in_no;
                self.ex_str = s.in_str.to_string();
                self.ex_bool0 = s.in_bool;
                self.last_command = SetValues(s,e);
                Ok(())
            },
            ExampleEnum::One => {
                self.ex_no1 = s.in_no;
                self.ex_str = s.in_str.to_string();
                self.ex_bool1 = s.in_bool;
                self.last_command = SetValues(s,e);
                Ok(())
            },
            _ => {
                self.last_command = SetValues(s,e);
                Err(ExampleError::SetErr)
            },
        }   
    }

    // I2C Example Transfer (Write-Read)
    // These functions serve as examples how to implement a write-read to payload via I2C
    // This is the preferred function used for commanding I2C payloads
    // Examples for Write and Read are given below for completeness
    // 
    // The I2C transfer function has the structure:
    // transfer(&self, command: Command, rx_len: usize, delay: Duration)
    // 
    pub fn get_i2c(&mut self) -> ExampleResult<Vec<u8>> {
        self.last_command = GetI2c;

        let cmd: u8 = I2C_GET;
        let rx_len = 1;
        let delay = Duration::from_millis(50);

        let data: Vec<u8> = Vec::new();
        let command = Command{cmd, data};

        match self.i2c_connection.transfer(command, rx_len, delay) {
            Ok(x) => Ok(x),
            Err(e) => Err(ExampleError::I2CError(e.kind())),
        }
    }
    pub fn set_i2c(&mut self, i: u8) -> ExampleResult<()> {
        self.last_command = SetI2c(i);

        let cmd: u8 = I2C_SET;
        let rx_len = 1;
        let delay = Duration::from_millis(50);

        let mut data: Vec<u8> = Vec::new();
        data.push(i);
        let command = Command{cmd, data};

        match self.i2c_connection.transfer(command, rx_len, delay) {
            Ok(x) => {
                if x[0] == cmd {
                    Ok(())
                } else {
                    Err(ExampleError::I2CSet)
                } 
            },               
            Err(e) => Err(ExampleError::I2CError(e.kind())),
        }
    }

    // This function serves as an example how to write a payload via I2C
    //
    // The I2C write function has the structure:
    // write(&self, command: Command)
    // 
    // pub fn i2c_write(&self, i: ExampleInput) -> ExampleResult<()> {
    //     let cmd: u8 = i.in_no as u8;

    //     if cmd != 0 {
    //         let data: Vec<u8> = Vec::new();
    //         data.push(i.in_str.to_vec());
    //         let command = Command{cmd, data};

    //         match self.connection.write(command) {
    //             Ok(()) => Ok(()),
    //             Err(_) => Err(ExampleError::Err),
    //         }
    //     }
    // }
    
    // I2C Example Read
    // This function serves as an example how to read from a payload via I2C
    //
    // The I2C read function has the structure:
    // read(&self, command: Command, rx_len: usize)
    // 
    // pub fn i2c_read(&self, cmd: Command) -> ExampleResult<ExampleOutput> {       
    //     let rx_len: usize = 10;
    //     match self.connection.read(cmd.cmd, rx_len) {
    //         Ok(x) => Ok(ExampleOutput{
    //                 out_no: x as u8,
    //                 out_str: "".to_string(),
    //                 out_bool: true,
    //             }),
    //         Err(_) => Err(ExampleError::Err),
    //     }                
    // }

    
    // UART Examples
    // These functions serves as an example how to communicate with a payload via UART
    // 
    // The UART read function has the structure:
    // read(&self, len: usize, timeout: Duration)
    // 
    // The UART write function has the structure:
    // write(&self, data: &[u8])
    // 

    pub fn get_uart(&mut self) -> ExampleResult<Vec<u8>> {
        self.last_command = GetUart;
        let get = &[UART_GET];

        match self.uart_connection.write(get) {
            Ok(_) => {
                // sleep 10ms
                thread::sleep(Duration::from_millis(10));

                // let mut buffer = self.buffer.borrow_mut();
                // Reads 1 byte, with a timeout of 1ms
                Ok(self.uart_connection.read(1, Duration::from_millis(1))?)
            }
            Err(e) => Err(ExampleError::UARTError(e)),
        }
    }

    // This example explores the possibilty of the payload not returning anything
    // If the UART payload sends a reply to your set-command, implementation is similar to the get_uart command
    pub fn set_uart(&mut self, input: u8) -> ExampleResult<()> {
        self.last_command = SetUart(input);
        let set = &[UART_SET, input];

        match self.uart_connection.write(set) {
            Ok(_) => Ok(()),
            Err(e) => Err(ExampleError::UARTError(e)),      
        }
    }
}