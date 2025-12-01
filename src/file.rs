//! ファイル/フォルダのエンコード・デコード機能

use crate::{decode_pnk, encode_pnk, pink072_unwrap, pink072_wrap};
use std::fs::{self, File};
use std::io::{self, Cursor, Read, Write};
use std::path::Path;
use zip::write::SimpleFileOptions;
use zip::{ZipArchive, ZipWriter};

/// Payload Type定義
pub const PAYLOAD_TYPE_RAW: u8 = 0;
pub const PAYLOAD_TYPE_FILE: u8 = 1;
pub const PAYLOAD_TYPE_ZIP: u8 = 2;

/// 単一ファイルをPNKにエンコード
///
/// ペイロード構造: `[ファイル名長 2B (LE)][ファイル名 UTF-8][データ]`
pub fn encode_file(
    input_path: &Path,
    output_path: &Path,
    seed9: &[u8; 9],
    strength: u8,
) -> io::Result<()> {
    let file_name = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid file name"))?;

    let file_data = fs::read(input_path)?;
    let payload = build_file_payload(file_name, &file_data);

    let frame = pink072_wrap(&payload, PAYLOAD_TYPE_FILE, seed9, strength)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let pnk = encode_pnk(&frame);
    fs::write(output_path, pnk)?;

    Ok(())
}

/// PNKから単一ファイルをデコード
pub fn decode_file(input_path: &Path, output_dir: &Path) -> io::Result<String> {
    let pnk_data = fs::read(input_path)?;
    let frame = decode_pnk(&pnk_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let (payload_type, payload) = pink072_unwrap(frame)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    if payload_type != PAYLOAD_TYPE_FILE {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected payload type {PAYLOAD_TYPE_FILE}, got {payload_type}"),
        ));
    }

    let (file_name, file_data) = parse_file_payload(&payload)?;

    fs::create_dir_all(output_dir)?;
    let output_path = output_dir.join(&file_name);
    fs::write(&output_path, file_data)?;

    Ok(file_name)
}

/// ファイル名とデータからペイロードを構築
fn build_file_payload(file_name: &str, data: &[u8]) -> Vec<u8> {
    let name_bytes = file_name.as_bytes();
    let name_len = name_bytes.len() as u16;

    let mut payload = Vec::with_capacity(2 + name_bytes.len() + data.len());
    payload.extend_from_slice(&name_len.to_le_bytes());
    payload.extend_from_slice(name_bytes);
    payload.extend_from_slice(data);
    payload
}

/// ペイロードからファイル名とデータを抽出
fn parse_file_payload(payload: &[u8]) -> io::Result<(String, &[u8])> {
    if payload.len() < 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "payload too short",
        ));
    }

    let name_len = u16::from_le_bytes([payload[0], payload[1]]) as usize;

    if payload.len() < 2 + name_len {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid file name length",
        ));
    }

    let file_name = String::from_utf8(payload[2..2 + name_len].to_vec())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid UTF-8 file name"))?;

    let data = &payload[2 + name_len..];
    Ok((file_name, data))
}

/// 生データをPNKにエンコード（ファイル名なし）
pub fn encode_raw(
    data: &[u8],
    output_path: &Path,
    seed9: &[u8; 9],
    strength: u8,
) -> io::Result<()> {
    let frame = pink072_wrap(data, PAYLOAD_TYPE_RAW, seed9, strength)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let pnk = encode_pnk(&frame);
    fs::write(output_path, pnk)?;

    Ok(())
}

/// PNKから生データをデコード
pub fn decode_raw(input_path: &Path) -> io::Result<Vec<u8>> {
    let pnk_data = fs::read(input_path)?;
    let frame = decode_pnk(&pnk_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let (payload_type, payload) = pink072_unwrap(frame)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    if payload_type != PAYLOAD_TYPE_RAW {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected payload type {PAYLOAD_TYPE_RAW}, got {payload_type}"),
        ));
    }

    Ok(payload)
}

/// フォルダをZIP化してPNKにエンコード
pub fn encode_folder(
    input_path: &Path,
    output_path: &Path,
    seed9: &[u8; 9],
    strength: u8,
) -> io::Result<()> {
    let zip_data = create_zip_from_folder(input_path)?;

    let frame = pink072_wrap(&zip_data, PAYLOAD_TYPE_ZIP, seed9, strength)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let pnk = encode_pnk(&frame);
    fs::write(output_path, pnk)?;

    Ok(())
}

/// PNKからフォルダをデコード（ZIP展開）
pub fn decode_folder(input_path: &Path, output_dir: &Path) -> io::Result<Vec<String>> {
    let pnk_data = fs::read(input_path)?;
    let frame = decode_pnk(&pnk_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let (payload_type, payload) = pink072_unwrap(frame)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    if payload_type != PAYLOAD_TYPE_ZIP {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("expected payload type {PAYLOAD_TYPE_ZIP}, got {payload_type}"),
        ));
    }

    extract_zip_to_folder(&payload, output_dir)
}

/// フォルダからZIPを作成
fn create_zip_from_folder(folder_path: &Path) -> io::Result<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = ZipWriter::new(&mut buffer);
        let options =
            SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        add_folder_to_zip(&mut zip, folder_path, folder_path, options)?;
        zip.finish()?;
    }
    Ok(buffer.into_inner())
}

/// フォルダを再帰的にZIPに追加
fn add_folder_to_zip<W: Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    base_path: &Path,
    current_path: &Path,
    options: SimpleFileOptions,
) -> io::Result<()> {
    for entry in fs::read_dir(current_path)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path
            .strip_prefix(base_path)
            .map_err(|e| io::Error::other(e.to_string()))?;
        let name = relative_path.to_string_lossy();

        if path.is_dir() {
            zip.add_directory(format!("{name}/"), options)?;
            add_folder_to_zip(zip, base_path, &path, options)?;
        } else {
            zip.start_file(name.to_string(), options)?;
            let mut file = File::open(&path)?;
            let mut data = Vec::new();
            file.read_to_end(&mut data)?;
            zip.write_all(&data)?;
        }
    }
    Ok(())
}

/// ZIPをフォルダに展開
fn extract_zip_to_folder(zip_data: &[u8], output_dir: &Path) -> io::Result<Vec<String>> {
    let cursor = Cursor::new(zip_data);
    let mut archive = ZipArchive::new(cursor)?;
    let mut extracted_files = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = output_dir.join(file.mangled_name());

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
            extracted_files.push(file.name().to_string());
        }
    }

    Ok(extracted_files)
}

/// ファイルまたはフォルダを自動判定してエンコード
pub fn encode_auto(
    input_path: &Path,
    output_path: &Path,
    seed9: &[u8; 9],
    strength: u8,
) -> io::Result<()> {
    if input_path.is_dir() {
        encode_folder(input_path, output_path, seed9, strength)
    } else {
        encode_file(input_path, output_path, seed9, strength)
    }
}

/// PNKを自動判定してデコード
pub fn decode_auto(input_path: &Path, output_dir: &Path) -> io::Result<Vec<String>> {
    let pnk_data = fs::read(input_path)?;
    let frame = decode_pnk(&pnk_data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    let (payload_type, payload) = pink072_unwrap(frame)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

    match payload_type {
        PAYLOAD_TYPE_RAW => {
            let output_file = output_dir.join("data.bin");
            fs::create_dir_all(output_dir)?;
            fs::write(&output_file, &payload)?;
            Ok(vec!["data.bin".to_string()])
        }
        PAYLOAD_TYPE_FILE => {
            let (file_name, file_data) = parse_file_payload(&payload)?;
            fs::create_dir_all(output_dir)?;
            let output_file = output_dir.join(&file_name);
            fs::write(&output_file, file_data)?;
            Ok(vec![file_name])
        }
        PAYLOAD_TYPE_ZIP => extract_zip_to_folder(&payload, output_dir),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown payload type: {payload_type}"),
        )),
    }
}
