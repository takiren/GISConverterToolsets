use anyhow::{anyhow, Ok, Result};
use nom::{
    bytes, character,
    error::{Error, ErrorKind},
    Err, IResult, Needed, ToUsize,
};
use std::collections::BTreeMap;

use std::io::Read;
use std::path::{Path, PathBuf};

pub struct XrainForm {
    ///地整識別
    owner: u8,
    ///データ種別3
    /// 1byte:対象エリアの地整識別コード
    mesh_kind: u16,
    ///観測日時(WIP)
    datetime: usize,
    response_status: u8,
    block_num: u16,
    data_size: u32,
    bottom_left: u16,
    top_right: u16,
}

pub struct XrainBinary<T>
where
    T: Sized,
{
    form: XrainForm,
    data: Vec<XrainDataBlock<T>>,
}

pub struct MeshCollection {
    primary_meshed: BTreeMap<u32, PrimaryMesh>,
}

pub struct PrimaryMesh {
    lat: u8,
    lon: u8,
    secondary_mesh: BTreeMap<u32, SecondaryMesh>,
}

pub struct SecondaryMesh {
    x: u8,
    y: u8,
    xrain_cells: Vec<XrainCell<u16>>,
}
pub struct XrainDataBlock<T> {
    cells: Vec<XrainCell<T>>,
}
pub struct XrainCell<T> {
    quality: T,
    strength: T,
}

struct XrainParser {
    meshes: MeshCollection,

    bin_data: Vec<u8>,
}

fn take_streaming<C>(i: &[u8], c: C) -> IResult<&[u8], &[u8]>
where
    C: ToUsize,
{
    bytes::streaming::take(c)(i)
}

fn take_complete(i: &[u8]) -> IResult<&[u8], &[u8]> {
    bytes::complete::take(1u8)(i)
}

impl XrainParser {
    fn read_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<u8>> {
        let mut file = std::fs::File::open(file_path).expect("file open failed");
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).expect("file read failed");
        Ok(buf)
    }

    fn read_header(bin_slice: &[u8]) -> Result<&[u8]> {
        let input = bin_slice;
        //固定値チェック:1byte
        println!("Checking 01");
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0xFD]);
        //地整識別チェック:1byte
        //TODO:チェックをどうするか。
        let (input, _extracted) = take_streaming(input, 1u8).unwrap();
        println!("Checking 02");
        //データ種別1:1byte
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x80]);
        //データ種別2:1byte
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x01]);
        //データ種別3:2byte
        let (input, _extracted) = take_streaming(input, 2u8).unwrap();
        //ヘッダ種別:1byte
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x01]);
        //観測値識別
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x05]);
        //観測日時
        let (input, _extracted) = take_streaming(input, 16u8).unwrap();

        //システムステータス
        let (input, _extracted) = take_streaming(input, 16u8).unwrap();

        //装置no.
        let (input, _extracted) = take_streaming(input, 1u8).unwrap();

        //11応答ステータス
        let (input, _extracted) = take_streaming(input, 1u8).unwrap();

        //ブロック数
        println!("Checking block num");
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        let mut earr: [u8; 2] = [0; 2];
        (0..2).for_each(|i| {
            earr[i] = extracted[i];
            println!("{}", extracted[i]);
        });
        println!("ブロック数 :{}", u16::from_be_bytes(earr));

        //データサイズ
        println!("Checking data size");

        let (input, extracted) = take_streaming(input, 4u8).unwrap();
        let mut earrr: [u8; 4] = [0; 4];
        (0..4).for_each(|i| {
            earrr[i] = extracted[i];
            println!("{}", extracted[i]);
        });
        println!("size :{}", u32::from_be_bytes(earrr));
        //南西端 bottom_left
        let (input, _extracted) = take_streaming(input, 2u8).unwrap();
        //北東端
        let (input, _extracted) = take_streaming(input, 2u8).unwrap();

        //予備をスキップ
        let (input, _extracted) = take_streaming(input, 10u8).unwrap();
        //固定値
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        assert_eq!(extracted, &[0x00, 0x00]);
        Ok(input)
    }

    fn read_block(&self, input: &[u8]) -> Result<(&[u8], &str)> {
        let (input, lat) = take_streaming(input, 1u8).unwrap();
        let (input, lon) = take_streaming(input, 1u8).unwrap();

        let prim_mesh_code = Into::<u32>::into(lat[0]) * 100 + Into::<u32>::into(lon[0]);

        let (input, mesh_code) = take_streaming(input, 1u8).unwrap();
        let grid_position: u8 = mesh_code[0];
        let ymask: u8 = 0b11110000;
        let xmask: u8 = 0b00001111;

        let xnum = grid_position & xmask;
        let ynum = grid_position & ymask >> 4;

        let mut i: u8 = 0;
        //連続するブロック数
        let (input, block_num) = take_streaming(input, 1u8).unwrap();
        //セル数は1から始まるので1を弾いてあげる。
        let limit = block_num[0] - 1;

        while i > limit {
            i += 1;
        }

        todo!()
    }

    fn read_cell(input: &[u8]) -> Result<(&[u8], XrainCell<u16>)> {
        let quality_mask: u16 = 0b1111000000000000;
        let rain_mask: u16 = 0b0000111111111111;
        let (out, _) = take_streaming(input, 2u8).unwrap();
        let mut cell_array: [u8; 2] = [0; 2];
        (0..2).for_each(|i| {
            cell_array[i] = out[i];
        });
        let val = u16::from_be_bytes(cell_array);
        let strength = val & rain_mask;
        let quality = val & quality_mask;
        let raincell = XrainCell { quality, strength };
        Ok((out, raincell))
    }
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_header_read() -> Result<()> {
        let raw = XrainParser::read_file("KANTO00001-20191011-0100-G000-EL000000")?;
        let input = raw.as_slice();
        //固定値チェック:1byte
        println!("Checking 01");
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0xFD]);
        //地整識別チェック:1byte
        //TODO:チェックをどうするか。
        let (input, _extracted) = take_streaming(input, 1u8).unwrap();
        println!("Checking 02");
        //データ種別1:1byte
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x80]);
        //データ種別2:1byte
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x01]);
        //データ種別3:2byte
        let (input, _extracted) = take_streaming(input, 2u8).unwrap();
        //ヘッダ種別:1byte
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x01]);
        //観測値識別
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0x05]);
        //観測日時
        let (input, _extracted) = take_streaming(input, 16u8).unwrap();

        //システムステータス
        let (input, _extracted) = take_streaming(input, 16u8).unwrap();

        //装置no.
        let (input, _extracted) = take_streaming(input, 1u8).unwrap();

        //11応答ステータス
        let (input, _extracted) = take_streaming(input, 1u8).unwrap();

        //ブロック数
        println!("Checking block num");
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        let mut earr: [u8; 2] = [0; 2];
        (0..2).for_each(|i| {
            earr[i] = extracted[i];
            println!("{}", extracted[i]);
        });
        println!("ブロック数 :{}", u16::from_be_bytes(earr));

        //データサイズ
        println!("Checking data size");

        let (input, extracted) = take_streaming(input, 4u8).unwrap();
        let mut earrr: [u8; 4] = [0; 4];
        (0..4).for_each(|i| {
            earrr[i] = extracted[i];
            println!("{}", extracted[i]);
        });
        println!("size :{}", u32::from_be_bytes(earrr));
        //南西端 bottom_left
        let (input, _extracted) = take_streaming(input, 2u8).unwrap();
        //北東端
        let (input, _extracted) = take_streaming(input, 2u8).unwrap();

        //予備をスキップ
        let (input, _extracted) = take_streaming(input, 10u8).unwrap();
        //固定値
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        assert_eq!(extracted, &[0x00, 0x00]);
        Ok(())
    }
}
