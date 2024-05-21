use std::fs::{File, OpenOptions};
use std::{io, env};
use std::io::{BufRead, BufReader, stdout, Write, BufWriter};
use std::path::Path;
use std::time::{Duration, Instant};
use base58::FromBase58;
use crate::color::{blue, green, magenta, red};
mod color;

const BACKSPACE: char = 8u8 as char;

fn main() {
    let version: &str = env!("CARGO_PKG_VERSION");
    println!("{}", blue("========================"));
    println!("{}{}", blue("ADDRESS TO H160 v:"), magenta(version));
    println!("{}", blue("========================"));

    // *******************************************
    // читаем файл с адресами и конвертируем их в h160 для базы
    // -----------------------------------------------------------------
    println!("{}", blue("Читаем файл с адресами и конвертируем их в h160"));

    let file_content = lines_from_file("address.txt").unwrap_or_else(|_| {
        let dockerfile = include_str!("address.txt");
        add_v_file("address.txt", dockerfile.to_string());
        lines_from_file("address.txt").expect("Ошибка чтения файла")
    });

    // создаём файл
    let file = match File::create("btc_h160.txt") {
        Ok(f) => f,
        Err(e) => panic!("Не удалось создать файл: {}", e),
    };
    let mut writer = BufWriter::new(file);

    // для измерения скорости
    let mut start = Instant::now();
    let mut speed: u32 = 0;
    let one_sec = Duration::from_secs(1);

    let len_file = file_content.len();
    let mut progress = len_file;

    // хешируем
    for (index, address) in file_content.iter().enumerate() {
        let binding = match address.from_base58() {
            Ok(value) => value,
            Err(_err) => {
                eprintln!("{}", red(format!("ОШИБКА ДЕКОДИРОВАНИЯ в base58 адрес: {}/строка: {}", address, index + 1)));
                continue; // Пропускаем этот адрес и переходим к следующему
            }
        };

        if binding.len() >= 21 {
            // измеряем скорость и отображаем прогресс
            speed += 1;
            progress -= 1;
            if start.elapsed() >= one_sec {
                let mut stdout = stdout();
                print!("{}\r{}", BACKSPACE, green(format!("|| СКОРОСТЬ: {}/сек || ОСТАЛОСЬ: {} шт ||", speed, progress)));
                stdout.flush().unwrap();
                start = Instant::now();
                speed = 0;
            }

            if let Err(e) = writeln!(writer, "{}", hex::encode(&binding.as_slice()[1..21])) {
                eprintln!("Не удалось записать в файл: {}", e);
            }
        } else {
            eprintln!("{}", red(format!("ОШИБКА, АДРЕС НЕ ВАЛИДЕН адрес: {}/строка: {}", address, index + 1)));
        }
    }
    writer.flush().expect("TODO: panic message");
    println!("{}","");
    let len_btc = get_lines("btc_h160.txt");
    println!("{}", blue(format!("Конвертировано адресов в h160: {}/{}", len_file, len_btc)));
    //-----------------------------------------------------------------------

    // Ожидание ввода пользователя для завершения программы
    println!("{}", blue("Нажмите Enter, чтобы завершить программу..."));
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Ошибка чтения строки");
}

fn lines_from_file(filename: impl AsRef<Path>) -> io::Result<Vec<String>> {
    BufReader::new(File::open(filename)?).lines().collect()
}

fn add_v_file(name: &str, data: String) {
    OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open(name)
        .expect("Не удалось открыть файл")
        .write_all(data.as_bytes())
        .expect("Запись не удалась");
}

fn get_lines(file: &str) -> usize {
    let file = File::open(file).expect("Не удалось открыть файл");
    let reader = BufReader::new(file);
    reader.lines().count()
}
