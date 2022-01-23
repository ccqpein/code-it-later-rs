#![feature(never_type)]

use datatypes::Bread;

pub mod args;
pub mod config;
pub mod datatypes;
pub mod fs_operation;

//:= TODO: can connect config with the prompt for implementing...
//:= the feature that only clean special lines &&...
//:= interact interface
pub fn prompt(mut conf: config::Config) -> Result<Option<()>, String> {
    if conf.delete {
        // only delete is true gonna triger the prompt
        let mut rl = rustyline::Editor::<()>::new();
        let readline = rl.readline("Are you sure you want to delete crumbs? (y/n/s/i): ");
        match readline {
            Ok(s) => match s.as_str() {
                "y" => {
                    fs_operation::handle_files(conf).for_each(|b| println!("{}", b));
                    Ok(None)
                }
                "n" => {
                    conf.delete = false;
                    fs_operation::handle_files(conf).for_each(|b| println!("{}", b));
                    Ok(None)
                }
                "s" => {
                    conf.delete = false; // set false first
                    let breads: Vec<Bread> = fs_operation::handle_files(conf).collect();
                    loop {
                        breads.iter().for_each(|b| println!("{}", b)); // show breads
                        let ask_again =
                            rl.readline("Are you sure you want to delete crumbs? (y/n/s): ");

                        match ask_again {
                            Ok(ag) => match ag.as_str() {
                                "y" => {
                                    breads
                                        .into_iter()
                                        .for_each(|b| fs_operation::clean_the_crumbs(b).unwrap());
                                    break;
                                }
                                "n" => break,
                                "s" => continue,
                                _ => {
                                    println!("I don't understand, please give y/n/s");
                                    break;
                                }
                            },
                            Err(e) => {
                                return Err(format!("error in prompt readline {}", e.to_string()))
                            }
                        }
                    }
                    Ok(None)
                }
                "i" => {
                    //:= Need tests
                    conf.delete = false; // set false first
                    let breads: Vec<Bread> = fs_operation::handle_files(conf).collect();
                    for b in breads {
                        println!("{}", b); // show breads
                        let ask_again = rl.readline(&format!(
                            "Do you want to delete {} 's crumb? (y/n): ",
                            b.file_path
                        ));

                        match ask_again {
                            Ok(ag) => match ag.as_str() {
                                "y" => fs_operation::clean_the_crumbs(b).unwrap(),
                                "n" => {
                                    continue;
                                }
                                _ => {
                                    println!("I don't understand, please give y/n");
                                    break;
                                }
                            },
                            Err(e) => {
                                return Err(format!("error in prompt readline {}", e.to_string()));
                            }
                        }
                    }
                    Ok(None)
                }
                _ => return Err("I don't understand, please give y/n/s".to_string()),
            },
            Err(e) => return Err(format!("error in prompt readline {}", e.to_string())),
        }
    } else {
        fs_operation::handle_files(conf).for_each(|b| println!("{}", b));
        Ok(None)
    }
}
