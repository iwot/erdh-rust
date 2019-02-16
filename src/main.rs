#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate mylib;
extern crate getopts;
use std::{env, process};
use getopts::Options;
use std::fs::File;
use std::io::{Write, BufWriter};

#[derive(Debug)]
struct Args {
  config_path: String,
  output_path: String,
}

fn print_usage(program: &str, opts: &Options) {
  let brief = format!("Usage: {} [options]", program);
  print!("{}", opts.usage(&brief));
  process::exit(0);
}

fn parse_args() -> Option<Args> {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optopt("c", "config", "set config path", "config.yaml");
    opts.optopt("o", "output", "set output path", "result.puml");
    opts.optflag("h", "help", "print this help menu");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };

    if matches.opt_present("h") {
        print_usage(&program, &opts);
        return None;
    }

    let config = matches.opt_str("c");
    let output = matches.opt_str("o");

    if let (Some(config_path), Some(output_path)) = (config, output) {
        Some(Args {
            config_path: config_path,
            output_path: output_path,
        })
    } else {
        print_usage(&program, &opts);
        None
    }
}

fn main() {
    let args = parse_args().unwrap();
    let config = mylib::config::Config::from_yaml_file(&args.config_path).unwrap();

    let mut cons = mylib::erdh::get_construction(&config).unwrap();
    if let Some(ex_info_path) = &config.ex_info {
        let ex_info = mylib::config::extra_config::ExtraConfig::from_yaml_file(&ex_info_path);
        if ex_info.is_ok() {
            mylib::erdh::apply_ex_info(&mut cons, &ex_info.unwrap());
        }
    }

    // 中間形式ファイルを保存
    if let Some(im) = config.intermediate {
        if let Some(save_to) = im.save_to {
            println!("saving intermediate data to {}", &save_to);
            let s = serde_yaml::to_string(&cons).unwrap();
            let mut writer_y = BufWriter::new(File::create(save_to).unwrap());
            writer_y.write_all(s.as_bytes()).unwrap();
            writer_y.flush().unwrap();
        }
    }

    // pumlを保存
    println!("saving plantuml data to {}", &args.output_path);
    let mut writer = BufWriter::new(File::create(args.output_path).unwrap());
    mylib::erdh::plantuml::write_puml(&cons, &mut writer, config.group).unwrap();
    writer.flush().unwrap();
}
