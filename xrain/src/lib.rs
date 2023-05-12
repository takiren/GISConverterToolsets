use anyhow::{anyhow, Ok, Result};
use nom::{
    bytes, character,
    error::{Error, ErrorKind},
    Err, IResult, Needed, ToUsize,
};
use std::{collections::BTreeMap, default};

use csv::Writer;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct XrainHeader {
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

impl Default for XrainHeader {
    fn default() -> Self {
        Self {
            owner: 71,
            mesh_kind: 0,
            datetime: 0,
            response_status: 0,
            block_num: 0,
            data_size: 0,
            bottom_left: 0,
            top_right: 0,
        }
    }
}

pub struct XrainBinary<T>
where
    T: Sized,
{
    form: XrainHeader,
    data: Vec<XrainDataBlock<T>>,
}

pub struct MeshCollection {
    primary_meshed: BTreeMap<u32, PrimaryMesh>,
}
impl Default for MeshCollection {
    fn default() -> Self {
        Self {
            primary_meshed: BTreeMap::new(),
        }
    }
}

#[derive(Debug)]
pub struct XrainBlockHeader {
    lat: u8,
    lon: u8,
    mesh_x: u8,
    mesh_y: u8,
    block_num: u8,
}

pub struct PrimaryMesh {
    lat: u8,
    lon: u8,
    secondary_mesh: BTreeMap<u32, SecondaryMesh>,
}

impl PrimaryMesh {
    fn new(lat: u8, lon: u8) -> Self {
        Self {
            lat,
            lon,
            secondary_mesh: BTreeMap::new(),
        }
    }
}

pub struct SecondaryMesh {
    //1次メッシュの下２桁
    primary_x: u8,
    //1次メッシュの上２桁
    primary_y: u8,
    x: u8,
    y: u8,
    xrain_cells: CellComposite,
}

pub struct CellComposite {
    cells: Vec<XrainCell<u16>>,
}

impl Default for CellComposite {
    fn default() -> Self {
        Self { cells: Vec::new() }
    }
}

impl CellComposite {
    fn push(&mut self, v: XrainCell<u16>) -> Result<()> {
        if self.cells.len() < 1600 {
            self.cells.push(v);
        } else {
            return Err(anyhow::anyhow!("Out of capacity"));
        }

        Ok(())
    }
}

impl SecondaryMesh {
    fn new(primary_x: u8, primary_y: u8, x: u8, y: u8, cells: CellComposite) -> Self {
        Self {
            primary_x,
            primary_y,
            x,
            y,
            xrain_cells: cells,
        }
    }

    fn assign_cells(&mut self, cell_composite: CellComposite) -> Result<()> {
        self.xrain_cells = cell_composite;
        Ok(())
    }

    fn save_csv<P: AsRef<Path>>(&self, out_path: P) -> Result<()> {
        let mut wtr = Writer::from_path(out_path)?;
        let xsize: usize = 40;
        let ysize: usize = 40;
        for i in 0..40 {
            let mut vline = Vec::<u16>::new();
            vline.reserve(40);
            for j in 0..40 {
                let index = i * 40 + j;
                vline.push(self.xrain_cells.cells.get(index).unwrap().strength);
            }

            wtr.serialize(vline)?;
        }
        wtr.flush()?;
        Ok(())
    }
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

impl Default for XrainParser {
    fn default() -> Self {
        Self {
            meshes: MeshCollection::default(),
            bin_data: Vec::new(),
        }
    }
}

impl XrainParser {
    fn read_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<u8>> {
        let mut file = std::fs::File::open(file_path).expect("file open failed");
        let mut buf: Vec<u8> = Vec::new();
        file.read_to_end(&mut buf).expect("file read failed");
        Ok(buf)
    }

    /// ヘッダーまで読み進めたスライスを返す（日本語正しいですか？)
    fn read_header(bin_slice: &[u8]) -> Result<(&[u8], XrainHeader)> {
        let mut header = XrainHeader::default();

        let input = bin_slice;
        //固定値チェック:1byte
        println!("Checking 01");
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        assert_eq!(extracted, &[0xFD]);
        //地整識別チェック:1byte
        //TODO:チェックをどうするか。
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        header.owner = extracted[0];

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
        //TODO:Impl datetime
        let (input, _extracted) = take_streaming(input, 16u8).unwrap();

        //システムステータス
        let (input, _extracted) = take_streaming(input, 16u8).unwrap();

        //装置no.
        let (input, _extracted) = take_streaming(input, 1u8).unwrap();

        //11応答ステータス
        let (input, extracted) = take_streaming(input, 1u8).unwrap();
        header.response_status = extracted[0];

        //ブロック数
        println!("Checking block num");
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        let mut earr: [u8; 2] = [0; 2];
        (0..2).for_each(|i| {
            earr[i] = extracted[i];
            println!("{}", extracted[i]);
        });
        let block_num = u16::from_be_bytes(earr);
        println!("ブロック数 :{}", block_num);
        header.block_num = block_num;

        //データサイズ
        println!("Checking data size");

        let (input, extracted) = take_streaming(input, 4u8).unwrap();
        //TODO:earrじゃなくてもっとましな名前を付ける。
        let mut earr: [u8; 4] = [0; 4];
        (0..4).for_each(|i| {
            earr[i] = extracted[i];
            println!("{}", extracted[i]);
        });
        let datasize = u32::from_be_bytes(earr);
        header.data_size = datasize;
        println!("size :{}", datasize);
        //南西端 bottom_left
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        let mut earr: [u8; 2] = [0; 2];
        for i in 0..2 {
            earr[i] = extracted[0];
        }
        let bl = u16::from_be_bytes(earr);
        println!("bottom left :{}", bl);
        header.bottom_left = bl;

        //北東端
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        let mut earr: [u8; 2] = [0; 2];
        for i in 0..2 {
            earr[i] = extracted[0];
        }
        let ur = u16::from_be_bytes(earr);
        println!("Upper right :{}", ur);
        //予備領域をスキップ
        let (input, _extracted) = take_streaming(input, 10u8).unwrap();
        //固定値
        let (input, extracted) = take_streaming(input, 2u8).unwrap();
        assert_eq!(extracted, &[0x00, 0x00]);
        Ok((input, header))
    }

    fn read_sequential_block<'a>(&self, input: &'a [u8]) -> Result<(&'a [u8], Vec<SecondaryMesh>)> {
        //緯度
        let (input, lat) = take_streaming(input, 1u8).unwrap();
        //経度
        let (input, lon) = take_streaming(input, 1u8).unwrap();

        //let prim_mesh_code = Into::<u32>::into(lat[0]) * 100 + Into::<u32>::into(lon[0]);

        //１次メッシュコード上２桁
        let lat = lat[0];
        //１次メッシュコード下２桁
        let lon = lon[0];

        let (input, mesh_code) = take_streaming(input, 1u8).unwrap();

        let grid_position: u8 = mesh_code[0];
        let ymask: u8 = 0b11110000;
        let xmask: u8 = 0b00001111;

        //西北、南北方向にそれぞれ８分割した位置
        //１次メッシュ内での経度位置(西から東,)
        let xnum = grid_position & xmask;
        //１次メッシュ内での緯度位置(南から北,)
        let ynum = (grid_position & ymask) >> 4;

        let mut i: u8 = 0;
        //連続するブロック数
        let (input, block_num) = take_streaming(input, 1u8).unwrap();
        //セル数は1から始まるので1を弾いてあげる。
        let block_num = block_num[0] - 1;
        let mut input = input;
        let mut v_smesh: Vec<SecondaryMesh> = Vec::new();
        while i < block_num {
            //先頭の２次メッシュコードに処理しているブロックのインデックスを足す。
            //それを８で割るとどこの１次メッシュに属しているかがわかる。
            //TODO:u8で足りるよね？考える
            let currentx = xnum + i;
            let currenty = ynum;
            let primary_x = lon + (currentx / 8);
            let primary_y = lat;
            let currentx = currentx % 8;
            let (input_internal, cmp) = XrainParser::read_single_block(input)?;
            input = input_internal;
            let smesh = SecondaryMesh::new(primary_x, primary_y, currentx, currenty, cmp);
            v_smesh.push(smesh);
            i += 1;
        }

        Ok((input, v_smesh))
    }

    fn read_single_block(input: &[u8]) -> Result<(&[u8], CellComposite)> {
        let mut cellcmp = CellComposite::default();
        //一つのブロックに入っているセルデータ数は40x40=1600
        for _i in 0..1600 {
            let (input, new_cell) = XrainParser::read_cell(input)?;
            cellcmp.push(new_cell)?;
        }
        Ok((input, cellcmp))
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

    fn read_block_header(input: &[u8]) -> Result<(&[u8], XrainBlockHeader)> {
        //緯度
        let (input, lat) = take_streaming(input, 1u8).unwrap();
        //経度
        let (input, lon) = take_streaming(input, 1u8).unwrap();

        //let prim_mesh_code = Into::<u32>::into(lat[0]) * 100 + Into::<u32>::into(lon[0]);

        //１次メッシュコード上２桁
        let lat = lat[0];
        //１次メッシュコード下２桁
        let lon = lon[0];

        let (input, mesh_code) = take_streaming(input, 1u8).unwrap();

        let grid_position: u8 = mesh_code[0];
        let ymask: u8 = 0b11110000;
        let xmask: u8 = 0b00001111;

        //西北、南北方向にそれぞれ８分割した位置
        //１次メッシュ内での経度位置(西から東,)
        let xnum = grid_position & xmask;
        //１次メッシュ内での緯度位置(南から北,)
        let ynum = (grid_position & ymask) >> 4;

        let mut i: u8 = 0;
        //連続するブロック数
        let (input, block_num) = take_streaming(input, 1u8).unwrap();

        let block_header = XrainBlockHeader {
            lat,
            lon,
            mesh_x: xnum,
            mesh_y: ynum,
            block_num: block_num[0],
        };

        Ok((input, block_header))
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

    #[test]
    fn test_read_single_block() -> Result<()> {
        let data = XrainParser::read_file("KANTO00001-20191011-0100-G000-EL000000")?;
        let (input, header) = XrainParser::read_header(data.as_slice())?;
        let (input, bheader) = XrainParser::read_block_header(input)?;
        println!("{:?}", header);
        let (input, cells) = XrainParser::read_single_block(input)?;
        
        println!("{:?}", bheader);
        Ok(())
    }
}
