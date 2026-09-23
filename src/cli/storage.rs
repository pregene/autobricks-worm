use crate::storage::{Record, Store};
use std::{
    io::{self, Read},
    path::Path,
};

fn show(record: &Record) -> io::Result<()> {
    serde_json::to_writer_pretty(io::stdout().lock(), record).map_err(io::Error::other)?;
    println!();
    Ok(())
}

pub fn run(args: &[String]) -> io::Result<()> {
    let [kind, root, operation, name, rest @ ..] = args else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Run ab-worm --help for usage",
        ));
    };
    if kind != "storage"
        || (operation == "create" && rest.len() != 1)
        || (operation != "create" && !rest.is_empty())
        || ![
            "create", "append", "read", "meta", "verify", "delete", "mkdir",
        ]
        .contains(&operation.as_str())
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid storage command",
        ));
    }
    let retention = if operation == "create" {
        let value = rest
            .first()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Retention is required"))?;
        Some(value.parse::<u64>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Retention must be an unsigned number of seconds",
            )
        })?)
    } else {
        None
    };
    let mut store = Store::open(Path::new(root))?;
    match operation.as_str() {
        "create" => {
            let retention = retention.ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "Retention is required")
            })?;
            show(&store.create(name, retention)?)
        }
        "append" => {
            let mut input = io::stdin().lock();
            let mut buffer = [0u8; 65536];
            loop {
                let length = input.read(&mut buffer)?;
                if length == 0 {
                    break;
                }
                let chunk = buffer.get(..length).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Input read exceeded buffer")
                })?;
                store.append(name, chunk)?;
            }
            show(&store.metadata(name)?)
        }
        "read" => {
            store.read(name, &mut io::stdout().lock())?;
            Ok(())
        }
        "meta" => show(&store.metadata(name)?),
        "verify" => {
            let record = store.verify(name)?;
            println!(
                "Verified: {name} ({} bytes, SHA-256 {})",
                record.lock_offset, record.checksum
            );
            Ok(())
        }
        "delete" => store.delete(name),
        "mkdir" => store.mkdir(name),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid storage command",
        )),
    }
}
