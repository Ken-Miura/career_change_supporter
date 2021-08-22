// Copyright 2021 Ken Miura

use std::{env::args, process::exit};

const INVALID_ARG_LENGTH: i32 = 1;
const INVALID_SUB_COMMAND: i32 = 2;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() < 2 {
        println!("usage: {} [ create | list | update | delete ] [sub_commarnd_args...]", args[0]);
        exit(INVALID_ARG_LENGTH);
    }
    let cmd = &args[1];
    if cmd == "create" {
    } else if cmd == "list" {
    } else if cmd == "update" {
    } else if cmd == "delete" {
    } else {
        println!("invalid subcommand: {}", cmd);
        println!("valid subcommand [ create | list | update | delete ]");
        exit(INVALID_SUB_COMMAND);
    }
}
