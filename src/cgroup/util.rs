//! Utility functions for manipulating the cgroup mount
//! Cgroup v2 reference: https://www.kernel.org/doc/Documentation/cgroup-v2.txt

use crate::error::ContainerErr;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, BufReader, Read};
use std::path::Path;

/// Reads existing data from a cgroup interface file which has a flat keyed format.
///
/// Example file data:
///
/// KEY0 VAL0\n
/// KEY1 VAL1\n
/// ...
///
pub fn read_flat_keyed_file<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, String>, ContainerErr> {
    let mut f = File::open(path).map_err(ContainerErr::IO)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf).map_err(ContainerErr::IO)?;

    let mut data = HashMap::new();

    for line in buf.split("\n") {
        let parts: Vec<&str> = line.split(" ").collect();
        if parts.len() == 2 {
            data.insert(String::from(parts[0]), String::from(parts[1]));
        }
    }

    Ok(data)
}

/// Reads existing data from a cgroup interface file which has values seprarated by spaces.
///
/// Example file data:
///
/// VAL0 VAL1 ...\n
///
pub fn read_space_separated_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>, ContainerErr> {
    let mut f = File::open(path).map_err(ContainerErr::IO)?;
    let mut buf = String::new();
    f.read_to_string(&mut buf).map_err(ContainerErr::IO)?;

    let slices: Vec<&str> = buf.split(" ").collect();
    let mut result = Vec::with_capacity(slices.len());
    for val in slices {
        result.push(String::from(val.trim()));
    }

    Ok(result)
}

/// Reads existing data from a cgroup interface file which has values seprarated by newlines.
///
/// Example file data:
///
/// VAL0\n
/// VAL1\n
/// ...
///
pub fn read_newline_separated_file<P: AsRef<Path>>(path: P) -> Result<Vec<String>, ContainerErr> {
    let mut data = Vec::new();

    let f = File::open(path).map_err(ContainerErr::IO)?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let line = line.map_err(ContainerErr::IO)?;
        data.push(line);
    }

    Ok(data)
}

/// Reads existing data from a cgroup interface file which has a nested key value format.
///
/// Example file data:
///
/// KEY0 SUB_KEY0=VAL00 SUB_KEY1=VAL01 ...
/// KEY1 SUB_KEY0=VAL10 SUB_KEY1=VAL11 ...
/// ...
///
pub fn read_nested_keyed_file<P: AsRef<Path>>(
    path: P,
) -> Result<HashMap<String, HashMap<String, String>>, ContainerErr> {
    let mut data = HashMap::new();

    let f = File::open(path).map_err(ContainerErr::IO)?;
    let reader = BufReader::new(f);

    for line in reader.lines() {
        let mut sub_map = HashMap::new();
        let line = line.map_err(ContainerErr::IO)?;

        let mut split = line.split(" ");
        let key = split.next();
        for sub_kv_pair in split {
            let pair = sub_kv_pair.split("=").collect::<Vec<&str>>();
            if pair.len() == 2 {
                sub_map.insert(String::from(pair[0]), String::from(pair[1]));
            }
        }

        if let Some(key) = key {
            data.insert(String::from(key), sub_map);
        }
    }

    Ok(data)
}

/// Writes to a cgroup interface file with a nested keyed format.
pub fn write_nested_keyed_file<P: AsRef<Path>>(
    path: P,
    data: HashMap<String, HashMap<String, String>>,
) -> Result<(), ContainerErr> {
    let mut s = String::new();
    for (k, v) in data.iter() {
        s += k;
        s += " ";
        for (sk, sv) in v.iter() {
            let pair = format!("{}={} ", &sk, &sv);
            s += &pair;
        }
        s.remove(s.len() - 1);
    }

    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(ContainerErr::IO)?;
    f.write_all(s.as_bytes()).map_err(ContainerErr::IO)?;
    Ok(())
}

/// Write to a cgroup interface file with a flat keyed format.
pub fn write_flat_keyed_file<P: AsRef<Path>>(
    path: P,
    data: HashMap<String, String>,
) -> Result<(), ContainerErr> {
    let mut s = String::new();
    for (k, v) in data.iter() {
        s += k;
        s += " ";
        s += v;
        s += "\n";
    }

    let mut f = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(path)
        .map_err(ContainerErr::IO)?;
    f.write_all(s.as_bytes()).map_err(ContainerErr::IO)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_read_nested_keyed_file() {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = format!("/tmp/read_nested_keyed_{}", time);

        {
            let data = b"KEY0 SUB0=VAL0 SUB1=VAL1\nKEY1 SUB11=VAL11";
            let mut tmp = File::create(&path).unwrap();
            tmp.write_all(data).unwrap();
        }

        let actual = read_nested_keyed_file(&path).unwrap();
        let mut expected = HashMap::new();
        let mut sm1 = HashMap::new();
        sm1.insert(String::from("SUB0"), String::from("VAL0"));
        sm1.insert(String::from("SUB1"), String::from("VAL1"));

        let mut sm2 = HashMap::new();
        sm2.insert(String::from("SUB11"), String::from("VAL11"));

        expected.insert(String::from("KEY0"), sm1);
        expected.insert(String::from("KEY1"), sm2);

        // Cleanup file
        std::fs::remove_file(&path).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_read_newline_file() {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = format!("/tmp/read_newline_{}", time);

        {
            let data = b"VAL0\nVAL1\n";
            let mut tmp = File::create(&path).unwrap();
            tmp.write_all(data).unwrap();
        }

        let actual = read_newline_separated_file(&path).unwrap();
        let expected = vec![String::from("VAL0"), String::from("VAL1")];

        // Cleanup file
        std::fs::remove_file(&path).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_read_space_separated_file() {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = format!("/tmp/read_space_separated_{}", time);

        {
            let data = b"VAL0 VAL1\n";
            let mut tmp = File::create(&path).unwrap();
            tmp.write_all(data).unwrap();
        }

        let actual = read_space_separated_file(&path).unwrap();
        let expected = vec![String::from("VAL0"), String::from("VAL1")];

        // Cleanup file
        std::fs::remove_file(&path).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_read_flat_keyed_file() {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = format!("/tmp/read_flat_keyed_{}", time);

        {
            let data = b"KEY0 VAL0\nKEY1 VAL1\n";
            let mut tmp = File::create(&path).unwrap();
            tmp.write_all(data).unwrap();
        }

        let actual = read_flat_keyed_file(&path).unwrap();
        let mut expected = HashMap::new();
        expected.insert(String::from("KEY0"), String::from("VAL0"));
        expected.insert(String::from("KEY1"), String::from("VAL1"));

        // Cleanup file
        std::fs::remove_file(&path).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_write_flat_keyed_file() {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let path = format!("/tmp/read_flat_keyed_{}", time);
        {
            let data = b"KEY20=VAL20\n";
            let mut tmp = File::create(&path).unwrap();
            tmp.write_all(data).unwrap();
            tmp.flush().unwrap();
            drop(tmp);
        }

        let mut data = HashMap::new();
        data.insert(String::from("KEY0"), String::from("VAL0"));
        data.insert(String::from("KEY1"), String::from("VAL1"));

        let expected = data.clone();
        write_flat_keyed_file(&path, data).unwrap();
        let actual = read_flat_keyed_file(&path).unwrap();

        // Cleanup file
        std::fs::remove_file(&path).unwrap();
        assert_eq!(expected, actual);
    }
}
