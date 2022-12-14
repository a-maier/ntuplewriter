mod bindings;
pub mod event;
pub mod ntuplewriter;
#[cfg(feature = "hepmc2")]
pub mod conv;

pub use crate::ntuplewriter::NTupleWriter;
pub use crate::event::Event;

include!(concat!(env!("OUT_DIR"), "/flags.rs"));

#[cfg(test)]
mod tests {
    use std::{process::Command, path::PathBuf, ffi::OsStr, fs::read_dir, os::unix::prelude::OsStrExt};
    use ntuplereader::NTupleReader;
    use tempfile::NamedTempFile;

    use super::*;

    fn read_event(reader: &mut NTupleReader) -> Option<Event> {
        if !reader.next_entry() {
            return None;
        }
        let nparticle = reader.get_particle_number();
        let event = Event {
            id: reader.get_id(),
            nparticle,
            px: (0..nparticle).map(|i| reader.get_x(i) as f32).collect(),
            py: (0..nparticle).map(|i| reader.get_y(i) as f32).collect(),
            pz: (0..nparticle).map(|i| reader.get_z(i) as f32).collect(),
            energy: (0..nparticle).map(|i| reader.get_energy(i) as f32).collect(),
            alphas: 0., // not supported by reader
            pdg_code: (0..nparticle).map(|i| reader.get_pdg_code(i)).collect(),
            weight: reader.get_weight(),
            weight2: reader.get_weight2(),
            me_weight: reader.get_me_weight(),
            me_weight2: reader.get_me_weight2(),
            x1: reader.get_x1(),
            x2: reader.get_x2(),
            x1p: 0., // not supported by reader
            x2p: 0., // not supported by reader
            id1: reader.get_id1() as i32,
            id2: reader.get_id2() as i32,
            fac_scale: reader.get_factorization_scale(),
            ren_scale: reader.get_renormalization_scale(),
            user_weights: vec![], // not supported by reader
            part: (reader.get_type() as u8 as char).try_into().unwrap(),
            alphas_power: reader.get_alphas_power(),
        };
        Some(event)
    }

    #[test]
    fn test() {
        let prefix = Command::new("nTupleReader-config")
            .arg("--prefix")
            .output()
            .unwrap()
            .stdout;
        let (_, prefix) = prefix.split_last().unwrap(); // remove newline
        let data_path = PathBuf::from_iter(
            [prefix, b"share", b"ntuplereader",].into_iter()
                .map(|p| OsStr::from_bytes(p))
        );
        for root_file in read_dir(data_path).unwrap() {
            let root_file = root_file.unwrap();

            let tmp1 = NamedTempFile::new().unwrap();
            let tmp2 = NamedTempFile::new().unwrap();

            let mut reader = NTupleReader::new();
            reader.add_file(root_file.path());
            {
                let mut writer = NTupleWriter::new(tmp1.path(), "").unwrap();
                while let Some(event) = read_event(&mut reader) {
                    writer.write(&event).unwrap();
                }
            }

            let mut reader = NTupleReader::new();
            reader.add_file(tmp1.path());
            {
                let mut writer = NTupleWriter::new(tmp2.path(), "").unwrap();
                while let Some(event) = read_event(&mut reader) {
                    writer.write(&event).unwrap();
                }
            }

            let mut reader1 = NTupleReader::new();
            reader1.add_file(tmp1.path());
            let mut reader2 = NTupleReader::new();
            reader2.add_file(tmp2.path());

            while let Some(event1) = read_event(&mut reader1) {
                let event2 = read_event(&mut reader2).unwrap();
                assert_eq!(event1, event2)
            }
        }
    }
}
