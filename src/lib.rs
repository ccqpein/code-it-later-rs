#![feature(never_type)]
#![feature(io_error_other)]

use std::collections::HashSet;

use datatypes::Bread;

pub mod args;
pub mod config;
pub mod datatypes;
pub mod fs_operation;

use datatypes::*;

//:= TODO:...
//:= delete single line;...
//:= interact prompt in single file delete;...
//:= emacs interact.
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

pub fn prompt_2(mut conf: config::Config) -> Result<Option<()>, String> {
    if conf.delete {
        // only delete is true gonna triger the prompt
        let mut rl = rustyline::Editor::<()>::new();
        conf.delete = false;
        let breads = fs_operation::handle_files(conf).collect::<Vec<_>>();
        loop {
            breads.iter().for_each(|b| println!("{}", b));
            match rl.readline("Are you sure you want to delete all crumbs? (y/n/s/i): ") {
                Ok(s) => match s.as_str() {
                    "y" => {
                        for b in breads {
                            fs_operation::clean_the_crumbs(b).map_err(|e| e.to_string())?
                        }
                    }
                    "n" => {} //do nothing
                    "s" => continue,
                    "i" => prompt_bread(breads.into_iter(), &mut rl)?,
                    _ => return Err("I don't understand, please give y/n/s/i".to_string()),
                },
                Err(e) => return Err(format!("error in prompt readline {}", e.to_string())),
            }
            break;
        }
        Ok(None)
    } else {
        fs_operation::handle_files(conf).for_each(|b| println!("{}", b));
        Ok(None)
    }
}

fn prompt_bread(
    breads: impl Iterator<Item = Bread>,
    rl: &mut rustyline::Editor<()>,
) -> Result<(), String> {
    for b in breads {
        loop {
            // incase need show again
            println!("{}", b);
            match rl.readline(&format!(
                "Are you sure you want to delete this bread {}? (y/n/s/i): ",
                b.file_path
            )) {
                Ok(s) => match s.as_str() {
                    "y" => fs_operation::clean_the_crumbs(b).map_err(|e| e.to_string())?,
                    "n" => {}
                    "s" => {
                        continue;
                    }
                    "i" => {
                        let go_to_delete = prompt_crumbs(b.crumbs.iter(), rl)?;
                        if go_to_delete.len() != 0 {
                            fs_operation::clean_the_crumbs_on_special_index(b, go_to_delete)
                                .unwrap()
                        }
                    }
                    _ => {
                        println!("I don't understand, please give y/n/s/i");
                    }
                },
                Err(e) => return Err(e.to_string()),
            }
            break;
        }
    }
    Ok(())
}

fn prompt_crumbs<'a>(
    crumbs: impl Iterator<Item = &'a Crumb>,
    rl: &mut rustyline::Editor<()>,
) -> Result<HashSet<usize>, String> {
    let mut going_to_delete_crumbs_indexes = HashSet::new();
    for (ind, c) in crumbs.enumerate() {
        loop {
            println!("{}", c);
            match rl.readline("Are you sure you want to delete this crumb? (y/n/s): ") {
                Ok(s) => match s.as_str() {
                    "y" => {
                        going_to_delete_crumbs_indexes.insert(ind);
                    }
                    "n" => {}
                    "s" => {
                        continue;
                    }
                    _ => {
                        println!("I don't understand, please give y/n/s");
                    }
                },
                Err(e) => return Err(e.to_string()),
            }
            break;
        }
    }
    Ok(going_to_delete_crumbs_indexes)
}
