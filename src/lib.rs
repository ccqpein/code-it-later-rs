#![feature(never_type)]
#![feature(io_error_other)]

use std::collections::HashSet;

use datatypes::Bread;

pub mod args;
pub mod config;
pub mod datatypes;
pub mod fs_operation;

use datatypes::*;

pub fn prompt(mut conf: config::Config) -> Result<Option<()>, String> {
    if conf.delete {
        //:= TODO: fmt only after delete...
        //:= if give the --fmt
        // only delete is true gonna triger the prompt
        let mut rl = rustyline::Editor::<()>::new();
        conf.delete = false;
        let breads = fs_operation::handle_files(conf).collect::<Vec<_>>();
        let mut maybe_need_to_format = None;
        loop {
            breads.iter().for_each(|b| println!("{}", b));
            match rl.readline("Are you sure you want to delete all crumbs? (y/n/s/i): ") {
                Ok(s) => match s.as_str() {
                    "y" => {
                        let mut cache = HashSet::new();
                        for b in breads {
                            cache.insert(
                                fs_operation::clean_the_crumbs(b).map_err(|e| e.to_string())?,
                            );
                        }
                        maybe_need_to_format = Some(cache);
                    }
                    "n" => (), //do nothing
                    "s" => continue,
                    "i" => maybe_need_to_format = Some(prompt_bread(breads.into_iter(), &mut rl)?),
                    _ => return Err("I don't understand, please give y/n/s/i".to_string()),
                },
                Err(e) => return Err(format!("error in prompt readline {}", e.to_string())),
            }
            break;
        }
        //:= format here
        Ok(None)
    } else {
        fs_operation::handle_files(conf).for_each(|b| println!("{}", b));
        Ok(None)
    }
}

fn prompt_bread(
    breads: impl Iterator<Item = Bread>,
    rl: &mut rustyline::Editor<()>,
) -> Result<HashSet<String>, String> {
    let mut maybe_need_to_format = HashSet::new();
    for b in breads {
        loop {
            // incase need show again
            println!("{}", b);
            match rl.readline(&format!(
                "Are you sure you want to delete this bread {}? (y/n/s/i): ",
                b.file_path
            )) {
                Ok(s) => match s.as_str() {
                    "y" => {
                        maybe_need_to_format
                            .insert(fs_operation::clean_the_crumbs(b).map_err(|e| e.to_string())?);
                    }
                    "n" => {}
                    "s" => {
                        continue;
                    }
                    "i" => {
                        let go_to_delete = prompt_crumbs(b.crumbs.iter(), rl)?;
                        if go_to_delete.len() != 0 {
                            maybe_need_to_format.insert(
                                fs_operation::clean_the_crumbs_on_special_index(b, go_to_delete)
                                    .map_err(|e| e.to_string())?,
                            );
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
    Ok(maybe_need_to_format)
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
