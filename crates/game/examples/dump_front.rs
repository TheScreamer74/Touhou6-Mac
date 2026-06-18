use std::fs; use std::path::PathBuf;
use th06_formats::anm0::Anm0; use th06_formats::pbg3::Pbg3;
fn main() {
    let dir = PathBuf::from("../TH06 ~ The Embodiment of Scarlet Devil/kouma");
    let data = fs::read(dir.join("CM.DAT")).unwrap();
    let arc = Pbg3::parse(&data).unwrap();
    let ent = arc.entries.iter().find(|e| e.name == "front.anm").unwrap();
    let anm = Anm0::parse(&arc.extract(ent).unwrap()).unwrap();
    let e = &anm.entries[0];
    println!("tex {}x{} sprites={} scripts={}", e.width, e.height, e.sprites.len(), e.scripts.len());
    for (id, instrs) in &e.scripts {
        // first sprite, scale, any op17 pos
        let mut sp=None; let mut scale=None; let mut pos=None; let mut ops=vec![];
        for i in instrs { ops.push(i.opcode);
            if i.opcode==1 && sp.is_none() && i.args.len()>=4 { sp=Some(u32::from_le_bytes([i.args[0],i.args[1],i.args[2],i.args[3]])); }
            if i.opcode==2 && scale.is_none() && i.args.len()>=8 { scale=Some((f32::from_le_bytes([i.args[0],i.args[1],i.args[2],i.args[3]]), f32::from_le_bytes([i.args[4],i.args[5],i.args[6],i.args[7]]))); }
            if i.opcode==17 && pos.is_none() && i.args.len()>=8 { pos=Some((f32::from_le_bytes([i.args[0],i.args[1],i.args[2],i.args[3]]), f32::from_le_bytes([i.args[4],i.args[5],i.args[6],i.args[7]]))); }
        }
        let rect = sp.and_then(|s| e.sprites.iter().find(|x|x.index==s)).map(|s|(s.x,s.y,s.width,s.height));
        println!("script {:2} sprite={:?} rect={:?} scale={:?} pos={:?} ops={:?}", id, sp, rect, scale, pos, ops);
    }
}
