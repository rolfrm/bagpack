extern crate chrono;
use std::io;
use chrono::offset::Utc;
use chrono::DateTime;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::env;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::ops::Add;
use std::path::Path;
use std::path::PathBuf;
use std::rc::Rc;
use tar::Archive;
use flate2::write::ZlibEncoder;
use flate2::Compression;

#[derive(Debug, Clone)]
struct FileInfo {
    path: String,
    len: u64,
    modified: std::time::SystemTime,
    is_dir: bool,
    dir: Option<Vec<FileInfo>>,
}

fn get_file_infos(dir: &Path) -> Vec<FileInfo> {
    let mut infos: Vec<FileInfo> = Vec::new();

    let entries = fs::read_dir(dir).unwrap();
    for entry in entries {
        if let Ok(entry) = entry {
            // Here, `entry` is a `DirEntry`.
            if let Ok(file_type) = entry.file_type() {
                let path = entry.path();
                let metadata = fs::metadata(&path).unwrap();
                let is_dir = file_type.is_dir();

                let mut f = FileInfo {
                    path: entry.file_name().into_string().unwrap(),
                    len: metadata.len(),
                    modified: metadata.modified().unwrap(),
                    is_dir: is_dir,
                    dir: None,
                };
                if is_dir {
                    f.dir = Some(get_file_infos(&path));
                }
                infos.push(f);
            }
        }
    }
    return infos;
}

fn write_string_to_file(file: &Path, to_write: String) -> Result<(), io::Error> {
    println!("Writing to {}", file.to_str().unwrap());
    let mut f = fs::File::create(file).unwrap();
    return f.write_all(to_write.as_bytes());
}

fn wstr_file(items : &mut Vec<Box<Path>>, path : PathBuf, data : String) -> Result<(), io::Error>{
    let mut f = fs::File::create(&path).unwrap();
    let p2 = path.into_boxed_path();
    items.push(p2);
    
    return f.write_all(data.as_bytes());
}
fn wstr_dir(items : &mut Vec<Box<Path>>, path : PathBuf) -> Result<(), io::Error>{
    let p2 = path.into_boxed_path();
    if fs::create_dir(&p2).is_ok() {
        items.push(p2);
    }
    return Ok(());
}

trait Flattenable{
    fn flatten(&self) -> Vec<FileInfo>;
}

fn iterate_files(files : Vec<FileInfo>,flattened:  &mut Vec<FileInfo>) 
{
    let mut queue : VecDeque<FileInfo> = VecDeque::new();
    let mut out : Vec<FileInfo> = Vec::new();
 
    files.iter().for_each(|f| queue.push_front(f.clone()));
    while queue.len() > 0 {
        let item = queue.pop_back().unwrap();
        flattened.push(item.clone());
        out.push(item.clone());
        match item.dir{
            Some(x) => x.iter().for_each(|f| queue.push_front(f.clone())),
            None => {}
        }
    }
}

fn iterate_files2<F>(files : &Vec<FileInfo>, fcn: &F, base: &Path)
    where F: Fn(&FileInfo, &Path) {
        for item in files{
            let subbuf = base.join(&item.path);
            let sub = subbuf.as_path();
            fcn(&item, &sub);
            if let Some(x) = &item.dir {
                iterate_files2(&x, fcn, &sub);
            }
        }
    }


    
impl Flattenable for Vec<FileInfo>{
    fn flatten (&self) -> Vec<FileInfo>{
        let mut out : Vec<FileInfo> = Vec::new();
        iterate_files(self.to_vec(), &mut out);
        return out;
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    for arg in args.iter() {
        println!("Argument: {}", arg)
    }
    let mut path: String = "./".to_string();
    match args.len() {
        2 => {
            path = args[1].clone();
        }
        _ => {}
    }



    let root = Path::new(&path).join(Path::new("test2")).into_boxed_path();
    let mut items : Vec<Box<Path>> = Vec::new();
    wstr_dir(&mut items, root.to_path_buf());
    wstr_file(&mut items, root.join("f"), String::from("asd"));
    wstr_file(&mut items, root.join("D"), String::from("asddddddd"));   

    get_file_infos(Path::new(&path)).iter().for_each(|x| println!("{:?}", x));

    wstr_dir(&mut items, root.join("dir")).unwrap();
    wstr_file(&mut items, root.join("dir/x"), String::from("_____"));
    get_file_infos(Path::new(&path)).iter().for_each(|x| println!("{:?}", x));

    wstr_dir(&mut items, root.join("dir2"));
    wstr_file(&mut items, root.join("dir2/x"), String::from("_____"));
    let files :Vec<FileInfo> = get_file_infos(Path::new(&path));//.iter().for_each(|x| println!("{:?}", x));
    let flattened = files.flatten();
    println!("File Count {}", flattened.len());
    let fi = File::create("archive.tar").unwrap();
    let enc = ZlibEncoder::new(fi, Compression::default());
    let mut tar = tar::Builder::new(enc);
    //iterate_files(files, |xx : &FileInfo| tar.append_path(Path::new(&xx.path)).unwrap());
    iterate_files2(&files, &|x, s| println!(">> {} {}", x.path, s.to_str().unwrap()), &root);

    //iterate_files2(&files, &|x, s| {
        
    //}, &root);


    println!("Files: {}", items.len());
    //items.iter().rev().for_each(|x| println!("{}", x.to_str().unwrap()));

    

    for item in items.iter().rev(){
        
        //tar.append_file(path: item, file: &mut fs::File)
        println!("{}", item.to_str().unwrap());
        fs::remove_file(&item).or_else(|_| {fs::remove_dir(&item)});
    }

    /*fs::create_dir(root);

    {
        let entries = get_file_infos(Path::new(&path));

        for entry in &entries {
            println!("{:?}", entry);
        }

        write_string_to_file(
            format!("{}/{}/c", path, entries[0].path),
            "Hello!".to_string(),
        ).unwrap();
    }
    {
        let entries = get_file_infos(Path::new(&path));

        for entry in &entries {
            println!("{:?}", entry);
        }

        write_string_to_file(
            format!("{}/{}/d", path, entries[0].path),
            "Hello!".to_string(),
        ).unwrap();
    }
    {
        let entries = get_file_infos(Path::new(&path));
        for entry in &entries {
            println!("{:?}", entry);
        }

        write_string_to_file(
            format!("{}/{}/dir2/e", path, entries[0].path),
            "Hello!".to_string(),
        ).unwrap();
    }
    {
        let entries = get_file_infos(Path::new(&path));
        for entry in &entries {
            println!("{:?}", entry);
        }

        write_string_to_file(
            format!("{}/{}/e", path, entries[0].path),
            "Hello!".to_string(),
        ).unwrap();
        
    }
    
    println!("Hello, world!");*/
}
