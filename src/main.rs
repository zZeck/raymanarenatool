use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;

use nom::sequence::tuple;
use nom::{IResult};
use nom::bytes::complete::{take_while};
use nom::combinator::map;
use nom::error::Error;
use nom::multi::count;
use nom::number::complete::{le_u32, le_u16, le_u8, le_f32};

//see families and object_type let next =
//code for linked list handling pattern
//see states for another style

struct TextureInfo {
    field0: u32,
    field4: u16,
    field6: u16,
    off_tempBuffer: u32,
    fieldC: u32,
    field10: u32,
    flags: u32,
    height_ : u16,
    width_: u16,
    height: u16,
    width: u16,
    currentScrollX: u32,
    currentScrollY: u32,
    textureScrollingEnabled: u32,
    alphaMask: u32,
    field30: u32,
    numMipmaps: u32,
    field38: u32,
    field3C: u32,
    field40: u32,
    field44: u32,
    field48: u8,
    flagsByte: u8,
    name: String
}

//may have to switch from usize that assumes main lvl file, to something else if data actually can be in other lvl files
struct Globals {
    off_actualWorld: usize,
    off_dynamicWorld: usize,
    off_inactiveDynamicWorld: usize,
    off_fatherSector: usize,
    off_firstSubMapPosition: usize,

    /* The following 7 values are the "Always" structure. The spawnable perso data is dynamically copied to these superobjects.
    There can be at most (num_always) objects of this type active in a level, and they get reused by other objects when they despawn. */
    num_always: u32,
    //LinkedList<Perso> spawnablePersos;
    off_always_reusableSO: usize, // There are (num_always) empty SuperObjects starting with this one.
    off_always_reusableUnknown1: usize, // (num_always) * 0x2c blocks
    off_always_reusableUnknown2: usize, // (num_always) * 0x4 blocks
}

struct ObjectType {
    name: String,
    unk1: u8,
    id: u8,
    unk2: u16
}

struct VisualMaterialTexture {
    offset: u32,
    off_texture: u32,
    textureOp: u8,
    shadingMode: u8,
    uvFunction: u8,
    scrollByte: u8,
    properties: u32,
    scrollX: f32,
    scrollY: f32,
    rotateSpeed: f32,
    rotateDirection: f32,
    currentScrollX: f32,
    currentScrollY: f32,
    blendIndex: u32
}

struct AnimatedTexture {
    off_texture: u32,
    time: f32,
    texture: TextureInfo
}

struct VisualMaterial {
    flags: u32,
    ambientCoef: (f32, f32, f32, f32),
    diffuseCoef: (f32, f32, f32, f32),
    specularCoef: (f32, f32, f32, f32),
    color: (f32, f32, f32, f32),
    off_animTextures_first: u32,
    off_animTextures_current: u32,
    num_animTextures: u16,
    properties: u8,
    num_textures_in_material: u32,
    textures: Vec<VisualMaterialTexture>,
    animTextures: Vec<AnimatedTexture>

}

struct Family {
    offset: u32,
    off_family_next: u32,
    off_family_prev: u32,
    off_family_hdr: u32,
    family_index: u32,
    name: String
}

struct Transition {
    offset: u32,
    off_transition_next: u32,
    off_targetState: u32,
    off_stateToGo: u32,
    linkingType: u8
    //3 more bytes unknown?
}

//loader {
// textures
// lightmapTextures
// objectTypes
// actualWorld
// dynamicWorld
// inactiveDynamicWorld
// fatherSector

fn main() {
    let mut mainlvlFile = File::open("").expect("open fail");
    let mut mainlvlbuffer = Vec::new();
    mainlvlFile.read_to_end(&mut mainlvlbuffer).expect("read fail");

    let mut mainptrfile = File::open("").expect("open fail");
    let mut mainptrBuffer = Vec::new();
    mainptrfile.read_to_end(&mut mainptrBuffer).expect("read fail");

    let mut fixlvlFile = File::open("").expect("open fail");
    let mut fixlvlBuffer = Vec::new();
    fixlvlFile.read_to_end(&mut fixlvlBuffer).expect("read fail");

    let mut fixPtrFile = File::open("").expect("open fail");
    let mut fixPtrBuffer = Vec::new();
    fixPtrFile.read_to_end(&mut fixPtrBuffer).expect("read fail");

    let lvlBuffers = [&fixlvlBuffer, &mainlvlbuffer];

    let (remain, num_textures_fix) = le_u32::<_, Error<_>>(&mainlvlbuffer[292..]).expect("died");

    let num_textures_total = 1024usize;
    let num_textures_lvl = num_textures_total - num_textures_fix as usize;

    let (pointery, pointer_count) = le_u32::<_, Error<_>>(mainptrBuffer.as_slice()).expect("died");
    let (nomore, pointer_tuples) = count(tuple((le_u32::<_, Error<_>>, le_u32)), pointer_count as usize)(pointery).expect("died");

    let blah: HashMap<usize, usize> = pointer_tuples.into_iter().map(|(x, y)| (y as usize, x as usize)).collect();

    let (remain, textures) = count(|x| texture(x, &mainlvlbuffer, &lvlBuffers, &blah), num_textures_lvl)(remain).expect("died");

    let (remain, cnt_texture_stuff) = count(le_u32::<_, Error<_>>, num_textures_total)(remain).expect("died");

    //for blah in textures.iter().filter_map(|x| x.as_ref()) {
    //    println!("{}", blah.name)
    //}

    let (remain, lightTex) = texture(remain, &mainlvlbuffer, &lvlBuffers, &blah).expect("died");

    let (remain, off_actualWorld) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_dynamicWorld) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_inactiveDynamicWorld) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_fatherSector) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_firstSubMapPosition) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, num_always) = le_u32::<_, Error<_>>(remain).expect("died");

    //linked list
    let (remain, off_persos_head) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_persos_tail) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_persos_num_elements) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, off_always_reusableSO) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_always_reusableUnknown1) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_always_reusableUnknown2) = le_u32::<_, Error<_>>(remain).expect("died");

    let globals = Globals {
        off_actualWorld: off_actualWorld as usize,
        off_dynamicWorld: off_dynamicWorld as usize,
        off_inactiveDynamicWorld: off_inactiveDynamicWorld as usize,
        off_fatherSector: off_fatherSector as usize,
        off_firstSubMapPosition: off_firstSubMapPosition as usize,
        num_always: num_always,
        off_always_reusableSO: off_always_reusableSO as usize,
        off_always_reusableUnknown1: off_always_reusableUnknown1 as usize,
        off_always_reusableUnknown2: off_always_reusableUnknown2 as usize
    };

    let (remain, objectTypes) = count(|x| object_type_list(x, &mainlvlbuffer), 3)(remain).expect("died");

    let (remain, off_light) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_characterLaunchingSoundEvents) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, off_collisionGeoObj) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_staticCollisionGeoObj) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, viewport) = le_u32::<_, Error<_>>(remain).expect("died");

    //linked list
    let (remain, off_unk_head) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_unk_tail) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_unk_num_elements) = le_u32::<_, Error<_>>(remain).expect("died");

    //linked list
    let (remain, off_families_head) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_families_tail) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_families_num_elements) = le_u32::<_, Error<_>>(remain).expect("died");

    //linked list
    let (remain, off_always_active_characters_head) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_always_active_characters_tail) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_always_active_characters_num_elements) = le_u32::<_, Error<_>>(remain).expect("died");

    //linked list
    let (remain, off_main_characters_head) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_main_characters_tail) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_main_characters_num_elements) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, unk_no_transit_pref) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, unk_sol0) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, unk_sol1) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, unk_sol2) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, cinematic_manager) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, unk0) = le_u8::<_, Error<_>>(remain).expect("died");
    let (remain, ipo_numRLI_tables) = le_u8::<_, Error<_>>(remain).expect("died");
    let (remain, unk1) = le_u16::<_, Error<_>>(remain).expect("died");

    let (remain, off_COL_taggedFacesTable) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, num_COL_maxTaggedFaces) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, off_collisionGeoObj2) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_staticCollisionGeoObj2) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, unk2_ptrsTableSound) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, num_ptrsTable) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, off_ptrsTable) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, off_internalStructure_first) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_internalStructure_last) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, num_unk) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, unk_first) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, unk_last) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, num_visual_materials) = le_u32::<_, Error<_>>(remain).expect("died");
    let (remain, off_array_visual_materials) = le_u32::<_, Error<_>>(remain).expect("died");

    let (remain, off_dynamic_so_list) = le_u32::<_, Error<_>>(remain).expect("died");

    //parse super objects
    //LinkedList<SuperObject>.ReadHeader(reader, off_dynamic_so_list);
    let dyn_so = &mainlvlbuffer[(off_dynamic_so_list as usize + 4)..];
    let (dyn_so, off_main_characters_head) = le_u32::<_, Error<_>>(dyn_so).expect("died");
    let (dyn_so, off_main_characters_tail) = le_u32::<_, Error<_>>(dyn_so).expect("died");
    let (dyn_so, off_main_characters_num_elements) = le_u32::<_, Error<_>>(dyn_so).expect("died");

    //parse visual materials
    let mtt = &mainlvlbuffer[(off_array_visual_materials as usize + 4)..];
    let (_, materials) = count(|x| visual_material(x, &mainlvlbuffer, &lvlBuffers, &blah), num_visual_materials as usize)(mtt).expect("died");

    //no transit

    //Read Families
    let fm = &mainlvlbuffer[(off_families_head as usize + 4)..];
    let (_, families) = count(|x| families(x, &mainlvlbuffer, &objectTypes), off_families_num_elements as usize)(fm).expect("died");


}

fn families<'a>(input: &'a[u8], buffer: &'a[u8], objectTypes: &Vec<Vec<ObjectType>>) -> IResult<&'a[u8], Family> {
    
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, off_family_next) = le_u32::<_, Error<_>>(input)?;
    let (input, off_family_prev) = le_u32::<_, Error<_>>(input)?;
    let (input, off_family_hdr) = le_u32::<_, Error<_>>(input)?;

    let (input, family_index) = le_u32::<_, Error<_>>(input)?;
    let name = objectTypes[0][family_index as usize].name.clone();

    let (input, off_state_head) = le_u32::<_, Error<_>>(input)?;
    let (input, off_state_tail) = le_u32::<_, Error<_>>(input)?;
    let (input, off_state_num_elements) = le_u32::<_, Error<_>>(input)?;

    let state = &buffer[(off_state_head as usize + 4)..];
    let (_, states) = count(|x| states(x, &buffer), off_state_num_elements as usize)(state)?;

    //list not parsed in raymap?
    let (input, off_preloadAnim_head) = le_u32::<_, Error<_>>(input)?;
    let (input, off_preloadAnim_tail) = le_u32::<_, Error<_>>(input)?;
    let (input, off_preloadAnim_num_elements) = le_u32::<_, Error<_>>(input)?;

    let (input, off_physical_list_default) = le_u32::<_, Error<_>>(input)?;

    let (input, off_objectLists_head) = le_u32::<_, Error<_>>(input)?;
    let (input, off_objectLists_tail) = le_u32::<_, Error<_>>(input)?;
    let (input, off_objectLists_num_elements) = le_u32::<_, Error<_>>(input)?;

    let objectList = &buffer[(off_objectLists_head as usize + 4)..];
    let (_, objectLists) = count(|x| object_Lists(x, &buffer), off_objectLists_num_elements as usize)(objectList)?;


    let (input, off_bounding_volume) = le_u32::<_, Error<_>>(input)?;

    let (input, _) = le_u32::<_, Error<_>>(input)?;
    let (input, _) = le_u8::<_, Error<_>>(input)?;
    let (input, _) = le_u8::<_, Error<_>>(input)?;

    let (input, animBank) = le_u8::<_, Error<_>>(input)?;
    let (input, properties) = le_u8::<_, Error<_>>(input)?;



    let next = &buffer[(off_family_next as usize + 4)..];
    Ok((next, Family {
        offset: offset as u32,
        off_family_next: off_family_next,
        off_family_prev: off_family_prev,
        off_family_hdr: off_family_hdr,
        family_index: family_index,
        name: name
    }))
}

fn object_Lists<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], u8> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, off_objList_next) = le_u32::<_, Error<_>>(input)?;

    let (input, off_objList_start) = le_u32::<_, Error<_>>(input)?;
    let (input, off_objList_2) = le_u32::<_, Error<_>>(input)?;
    let (input, num_entries) = le_u16::<_, Error<_>>(input)?;
    let (input, _) = le_u16::<_, Error<_>>(input)?;

    let objectListEntr = &buffer[(off_objList_start as usize + 4)..];
    let (_, objectListEntry) = count(|x| object_List_entry(x, &buffer), num_entries as usize)(objectListEntr)?;

    Ok((input, 0))
}

fn object_List_entry<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], u8> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, off_scale) = le_u32::<_, Error<_>>(input)?;
    let (input, off_po) = le_u32::<_, Error<_>>(input)?;
    let (input, thirdvalue) = le_u32::<_, Error<_>>(input)?;
    let (input, unk0) = le_u16::<_, Error<_>>(input)?;
    let (input, unk1) = le_u16::<_, Error<_>>(input)?;

    let (input, lastValue) = le_u32::<_, Error<_>>(input)?;

    let (input, x) = if lastValue != 0 && thirdvalue != 0 {
        let sca = &buffer[(off_scale as usize + 4)..];
        //stream byte order is x z y
        let (_, scale) = tuple((le_f32, le_f32, le_f32))(sca)?;

        let sca = &buffer[(off_po as usize + 4)..];
        let (input, po) = physical_object(input, buffer)?;

        (input, Some((scale, po)))
    } else {
        (input, None)
    };

    Ok((input, 0))
}

struct VisualSetLOD {
    LODdistance: f32,
    off_data: u32
    //geometric obj?
}

fn physical_object<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], u8> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, off_visualSet) = le_u32::<_, Error<_>>(input)?;
    let (input, off_collideSet) = le_u32::<_, Error<_>>(input)?;
    let (input, off_visualBoundingVolume) = le_u32::<_, Error<_>>(input)?;

    let (input, off_collideBoundingVolume) = le_u32::<_, Error<_>>(input)?;

    let visSet = &buffer[(off_visualSet as usize + 4)..];
    let (visSet, _) = le_u32::<_, Error<_>>(visSet)?;
    let (visSet, numberOfLOD) = le_u16::<_, Error<_>>(visSet)?;
    let (visSet, visualSetType) = le_u16::<_, Error<_>>(visSet)?;

    let (visSet, x) = if numberOfLOD > 0 {
        let (visSet, off_LODDistances) = le_u32::<_, Error<_>>(visSet)?;
        let (visSet, off_LODDataOffsets) = le_u32::<_, Error<_>>(visSet)?;
        let lod = &buffer[(off_LODDistances as usize + 4)..];

        let (_, lods) = count(le_f32, numberOfLOD as usize)(lod)?;

        let loddat = &buffer[(off_LODDataOffsets as usize + 4)..];
        let (_, loddats) = count(le_u32, numberOfLOD as usize)(loddat)?;

        let fff = loddats.into_iter().map(|off_data| {
            let dat = &buffer[(off_data as usize + 4)..];
            let (_, obj) = match visualSetType {
                0 => geometric_object(dat, buffer),
                //1 => 1,
                _ => Ok((dat, 0))
            }.expect("died");// clean this up. probably some Nom map instead of iter map. lift to Result before op
            (off_data, obj)
        });

        let blah = lods.into_iter().zip(fff).map(|(lod, (off, dat))| VisualSetLOD {LODdistance: lod, off_data: off}).collect();

        (visSet, blah)
    } else {
        (visSet, Vec::new())
    };


    Ok((input, 0))
}

fn geometric_object<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], u8> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, off_vertices) = le_u32::<_, Error<_>>(input)?;
    let (input, off_normals) = le_u32::<_, Error<_>>(input)?;

    let (input, off_blendWeights) = le_u32::<_, Error<_>>(input)?;

    let (input, _) = le_u32::<_, Error<_>>(input)?;

    let (input, off_element_types) = le_u32::<_, Error<_>>(input)?;
    let (input, off_elements) = le_u32::<_, Error<_>>(input)?;

    let (input, _) = le_u32::<_, Error<_>>(input)?;

    let (input, off_parallelBoxes) = le_u32::<_, Error<_>>(input)?;

    let (input, lookAtMode) = le_u32::<_, Error<_>>(input)?;
    let (input, num_vertices) = le_u16::<_, Error<_>>(input)?;
    let (input, num_elements) = le_u16::<_, Error<_>>(input)?;
    let (input, _) = le_u16::<_, Error<_>>(input)?;
    let (input, num_parallelBoxes) = le_u16::<_, Error<_>>(input)?;
    let (input, sphereRadius) = le_f32::<_, Error<_>>(input)?;
    //x z y in file
    //new Vector3(sphereX, sphereY, sphereZ);
    let (input, sphereCenter) = tuple((le_f32, le_f32, le_f32))(input)?;
    let (input, _) = le_u32::<_, Error<_>>(input)?;
    let (input, _) = le_u32::<_, Error<_>>(input)?;
    let (input, _) = le_u16::<_, Error<_>>(input)?;

    let vert = &buffer[(off_vertices as usize + 4)..];
    let (_, vertices) = count(tuple((le_f32, le_f32, le_f32)), num_vertices as usize)(vert)?;

    let norm = &buffer[(off_normals as usize + 4)..];
    let (_, normals) = count(tuple((le_f32, le_f32, le_f32)), num_vertices as usize)(norm)?;

    //validate this later, family phys obj geo objs don't have this for shrine.
    let x = if off_blendWeights != 0 {
        let weight = &buffer[(off_blendWeights as usize + 4)..];
        //float[4][num_vertices]
        let (_, blendWeights) = count(|x| {
            let (x, off) = le_u32::<_, Error<_>>(x)?;
            let w = &buffer[(off as usize + 4)..];
            let (_, weights) = count(le_f32, num_vertices as usize)(w)?;
            let (x, _) = le_u32::<_, Error<_>>(x)?;
            let (x, _) = le_u32::<_, Error<_>>(x)?;
            let (x, _) = le_u16::<_, Error<_>>(x)?;
            let (x, _) = le_u16::<_, Error<_>>(x)?;
            Ok((x, weights))
        }, 4 as usize)(weight)?;

        blendWeights
    } else {
        Vec::new()
    };

    let elem_typs = &buffer[(off_element_types as usize + 4)..];
    let (_, element_types) = count(le_u16, num_elements as usize)(elem_typs)?;

    let elem = &buffer[(off_elements as usize + 4)..];
    let (_, elements) = count(|x| {
        let (u, off) = le_u32::<_, Error<_>>(x)?;
        let elem = &buffer[(off as usize + 4)..];
        Ok((x, 0))
    }, num_elements as usize)(elem)?;

    Ok((input, 0))
}

struct State {
    offset: u32,
    //family_index: u32,
    //index: u32
    off_state_next: u32,
    off_anim_ref: u32,
    transitions: Vec<Transition>,
    prohibitStates: Vec<Prohibit>,
    off_nextState: u32,
    off_mechanicsIdCard: u32,
    off_cine_mapname: u32,
    off_cine_name: u32,
    speed: u8,
    customStateBits: u8,
    mechanicsIDCard: Option<MechanicsIdCard>,
    cine_mapname: Option<String>,
    cine_name: Option<String>,
    anim_ref: AnimationReference
}

fn states<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], State> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;
    let (input, off_state_next) = le_u32::<_, Error<_>>(input)?;

    let (input, off_anim_ref) = le_u32::<_, Error<_>>(input)?;

    //list stuff
    let (input, off_transitions_head) = le_u32::<_, Error<_>>(input)?;
    let (input, off_transitions_tail) = le_u32::<_, Error<_>>(input)?;
    let (input, off_transitions_num_elements) = le_u32::<_, Error<_>>(input)?;
    let transition = &buffer[(off_transitions_head as usize + 4)..];
    let (_, transitions) = count(|x| transitions(x, &buffer), off_transitions_num_elements as usize)(transition)?;

    //list stuff
    let (input, off_prohibitStates_head) = le_u32::<_, Error<_>>(input)?;
    let (input, off_prohibitStates_tail) = le_u32::<_, Error<_>>(input)?;
    let (input, off_prohibitStates_num_elements) = le_u32::<_, Error<_>>(input)?;
    let prohibit = &buffer[(off_prohibitStates_head as usize + 4)..];
    let (_, prohibitStates) = count(|x| prohibitStates(x, &buffer), off_prohibitStates_num_elements as usize)(prohibit)?;

    let (input, off_nextState) = le_u32::<_, Error<_>>(input)?;
    let (input, off_mechanicsIDCard) = le_u32::<_, Error<_>>(input)?;

    let (input, off_cine_mapname) = le_u32::<_, Error<_>>(input)?;
    let (input, off_cine_name) = le_u32::<_, Error<_>>(input)?;

    let (input, _) = le_u8::<_, Error<_>>(input)?;
    let (input, speed) = le_u8::<_, Error<_>>(input)?;
    let (input, _) = le_u8::<_, Error<_>>(input)?;
    let (input, customStateBits) = le_u8::<_, Error<_>>(input)?;

    //is this really optional in arena
    let (input, mechanicsIDCard) = if off_mechanicsIDCard == 0 {
        let idCard = &buffer[(off_mechanicsIDCard as usize + 4)..];
        let (input, mechanicsIDCard) = mechanicsIdCard(idCard, buffer)?;
        (input,Some(mechanicsIDCard))
    } else {
        (input, None)
    };

    //not sure cinemapname is ever populated in arena, but it does need handled
    let (input, cine_mapname) = if off_cine_mapname == 0 {
        let n = &buffer[(off_cine_mapname as usize + 4)..];
        let (input, name) = map(take_while(|c| c != 0), |cs: &[u8]| String::from_utf8_lossy(cs).into_owned())(n)?;
        (input, Some(name))
    } else {
        (input, None)
    };

    //not sure cine_name is ever populated in arena, but it does need handled
    let (input, cine_name) = if off_cine_name == 0 {
        let n = &buffer[(off_cine_name as usize + 4)..];
        let (input, name) = map(take_while(|c| c != 0), |cs: &[u8]| String::from_utf8_lossy(cs).into_owned())(n)?;
        (input, Some(name))
    } else {
        (input, None)
    };

    let anim = &buffer[(off_anim_ref as usize + 4)..];
    let (input, anim_ref) = animationReference(anim, buffer)?;

    let next = &buffer[(off_state_next as usize + 4)..];
    Ok((next, State {
        offset: offset as u32,
        off_state_next: off_state_next,
        off_anim_ref: off_anim_ref,
        transitions: transitions,
        prohibitStates: prohibitStates,
        off_nextState: off_nextState,
        off_mechanicsIdCard: off_mechanicsIDCard,
        off_cine_mapname: off_cine_mapname,
        off_cine_name: off_cine_name,
        speed: speed,
        customStateBits: customStateBits,
        mechanicsIDCard: mechanicsIDCard,
        cine_mapname: cine_mapname,
        cine_name: cine_name,
        anim_ref: anim_ref
    }))
}

struct AnimationReference {
    offset: u32,
    num_onlyFrames: u16,
    speed: u8,
    num_channels: u8,
    off_events: u32,
    off_morphData: u32,
    anim_index: u16,
    num_events: u8,
    transition: u8
}

fn animationReference<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], AnimationReference> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, num_onlyFrames) = le_u16::<_, Error<_>>(input)?;
    let (input, speed) = le_u8::<_, Error<_>>(input)?;
    let (input, num_channels) = le_u8::<_, Error<_>>(input)?;
    let (input, off_events) = le_u32::<_, Error<_>>(input)?;

    let (input, off_morphData) = le_u32::<_, Error<_>>(input)?;

    let (input, anim_index) = le_u16::<_, Error<_>>(input)?;
    let (input, num_events) = le_u8::<_, Error<_>>(input)?;
    let (input, transition) = le_u8::<_, Error<_>>(input)?;

    //off_a3d and AnimA3DGeneral seems to not apply to arena

    Ok((input, AnimationReference {
        offset: offset as u32,
        num_onlyFrames: num_onlyFrames,
        speed: speed,
        num_channels: num_channels,
        off_events: off_events,
        off_morphData: off_morphData,
        anim_index: anim_index,
        num_events: num_events,
        transition: transition
    }))
}

struct MechanicsIdCard {
    offset: u32,
    idtype: u32,
    flags: u32,
    gravity: f32,
    maxRebound: f32,
    slopeLimit: f32,
    inertia: (f32, f32, f32),
    tiltIntensity: f32,
    tiltInertia: f32,
    tiltOrigin: f32,
    maxInertia: (f32, f32, f32)
}

fn mechanicsIdCard<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], MechanicsIdCard> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, idtype) = le_u32::<_, Error<_>>(input)?;
    let (input, flags) = le_u32::<_, Error<_>>(input)?;
    let (input, gravity) = le_f32::<_, Error<_>>(input)?;
    let (input, maxRebound) = le_f32::<_, Error<_>>(input)?;
    //tuple is in file as x z y
    let (input, _) = le_u32::<_, Error<_>>(input)?;
    let (input, slopeLimit) = le_f32::<_, Error<_>>(input)?;
    let (input, inertia) = tuple((le_f32, le_f32, le_f32))(input)?;
    let (input, tiltIntensity) = le_f32::<_, Error<_>>(input)?;
    let (input, tiltInertia) = le_f32::<_, Error<_>>(input)?;
    let (input, tiltOrigin) = le_f32::<_, Error<_>>(input)?;
    //tuple is in file as x z y
    let (input, maxInertia) = tuple((le_f32, le_f32, le_f32))(input)?;

    Ok((input, MechanicsIdCard {
        offset: offset as u32,
        idtype: idtype,
        flags: flags,
        gravity: gravity,
        maxRebound: maxRebound,
        slopeLimit: slopeLimit,
        inertia: inertia,
        tiltIntensity: tiltIntensity,
        tiltInertia: tiltInertia,
        tiltOrigin: tiltOrigin,
        maxInertia: maxInertia
    }))
}

struct Prohibit {
    offset: u32,
    off_prohibit_next: u32,
    off_state: u32
}

//this is never used in shrine
fn prohibitStates<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], Prohibit> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, off_prohibit_next) = le_u32::<_, Error<_>>(input)?;

    let (input, off_state) = le_u32::<_, Error<_>>(input)?;

    let next = &buffer[(off_prohibit_next as usize + 4)..];
    Ok((next, Prohibit {
        offset: offset as u32,
        off_prohibit_next: off_prohibit_next,
        off_state: off_state
    }))
}

fn transitions<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], Transition> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;

    let (input, off_transition_next) = le_u32::<_, Error<_>>(input)?;

    let (input, off_targetState) = le_u32::<_, Error<_>>(input)?;
    let (input, off_stateToGo) = le_u32::<_, Error<_>>(input)?;
    let (input, linkingType) = le_u8::<_, Error<_>>(input)?;

    let (input, _) = le_u8::<_, Error<_>>(input)?;
    let (input, _) = le_u8::<_, Error<_>>(input)?;
    let (input, _) = le_u8::<_, Error<_>>(input)?;

    let next = &buffer[(off_transition_next as usize + 4)..];
    Ok((next, Transition {
        offset: offset as u32,
        off_transition_next: off_transition_next,
        off_targetState: off_targetState,
        off_stateToGo: off_stateToGo,
        linkingType: linkingType
    }))
}

fn visual_material<'a>(input: &'a[u8], buffer: &'a[u8], buffers: &'a[&Vec<u8>; 2], pointers: &HashMap<usize, usize>) -> IResult<&'a[u8], VisualMaterial> {
    let (input, off_material) = le_u32::<_, Error<_>>(input)?;
    let mtt = &buffer[(off_material as usize + 4)..];
    let (mtt, flags) = le_u32::<_, Error<_>>(mtt)?;
    //tuples need reversed to match order in raymap
    let (mtt, ambientCoef) = tuple((le_f32, le_f32, le_f32, le_f32))(mtt)?;
    let (mtt, diffuseCoef) = tuple((le_f32, le_f32, le_f32, le_f32))(mtt)?;
    let (mtt, specularCoef) = tuple((le_f32, le_f32, le_f32, le_f32))(mtt)?;
    let (mtt, color) = tuple((le_f32, le_f32, le_f32, le_f32))(mtt)?;

    let (mtt, refres_num) = le_u32::<_, Error<_>>(mtt)?;

    let (mtt, off_animTextures_first) = le_u32::<_, Error<_>>(mtt)?;
    let (mtt, off_animTextures_current) = le_u32::<_, Error<_>>(mtt)?;
    let (mtt, num_animTextures) = le_u16::<_, Error<_>>(mtt)?;

    let (mtt, unk0) = le_u16::<_, Error<_>>(mtt)?;
    let (mtt, unk1) = le_u32::<_, Error<_>>(mtt)?;
    let (mtt, unk2) = le_u8::<_, Error<_>>(mtt)?;
    let (mtt, unk3) = le_u8::<_, Error<_>>(mtt)?;

    let (mtt, properties) = le_u8::<_, Error<_>>(mtt)?;

    let (mtt, unk4) = le_u8::<_, Error<_>>(mtt)?;
    let (mtt, unk5) = le_u32::<_, Error<_>>(mtt)?;
    let (mtt, unk6) = le_u32::<_, Error<_>>(mtt)?;

    let (mtt, num_textures_in_material) = le_u32::<_, Error<_>>(mtt)?;

    //why is this always 4, and num_texuters_in_material is not used
    let (mtt, textures) = count(|x| visual_material_texture(x, buffer, buffers, pointers), 4)(mtt)?;

    let num_textures = textures.len();

    let animt = &buffer[(off_animTextures_first as usize + 4)..];
    let (_, animTextures) = count(|x| anim_texture(x, buffer, buffers, pointers), num_animTextures as usize)(animt)?;

    Ok((input, VisualMaterial {
        flags: flags,
        ambientCoef: ambientCoef,
        diffuseCoef: diffuseCoef,
        specularCoef: specularCoef,
        color: color,
        off_animTextures_first: off_animTextures_first,
        off_animTextures_current: off_animTextures_current,
        num_animTextures: num_animTextures,
        properties: properties,
        num_textures_in_material: num_textures_in_material,
        textures: textures,
        animTextures: animTextures
    }))
}

fn anim_texture<'a>(input: &'a[u8], buffer: &'a[u8], buffers: &'a[&Vec<u8>; 2], pointers: &HashMap<usize, usize>) -> IResult<&'a[u8], AnimatedTexture> {
    let (input2, off_animTexture) = le_u32::<_, Error<_>>(input)?;
    let (_, time) = le_f32::<_, Error<_>>(input2)?;

    let (_, tex) = texture(input, &buffer, buffers, pointers)?;

    Ok((input, AnimatedTexture {
        off_texture: off_animTexture,
        time: time,
        texture: tex.unwrap()
    }))
}

fn visual_material_texture<'a>(input: &'a[u8], buffer: &'a[u8], buffers: &'a[&Vec<u8>; 2], pointers: &HashMap<usize, usize>) -> IResult<&'a[u8], VisualMaterialTexture> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let offset = inptr - buffPtr - 4;
    let (input2, off_texture) = le_u32::<_, Error<_>>(input)?;
    //my texture code reads the offset itself currently
    let (remain, tex) = texture(input, &buffer, buffers, pointers)?;

    let (input2, textureOp) = le_u8::<_, Error<_>>(input2)?;
    let (input2, shadingMode) = le_u8::<_, Error<_>>(input2)?;
    let (input2, uvFunction) = le_u8::<_, Error<_>>(input2)?;
    let (input2, scrollByte) = le_u8::<_, Error<_>>(input2)?;

    let (input2, properties) = le_u32::<_, Error<_>>(input2)?;

    let (input2, _) = le_u32::<_, Error<_>>(input2)?;
    let (input2, _) = le_u32::<_, Error<_>>(input2)?;

    let (input2, scrollX) = le_f32::<_, Error<_>>(input2)?;
    let (input2, scrollY) = le_f32::<_, Error<_>>(input2)?;
    let (input2, rotateSpeed) = le_f32::<_, Error<_>>(input2)?;
    let (input2, rotateDirection) = le_f32::<_, Error<_>>(input2)?;

    let (input2, _) = le_u32::<_, Error<_>>(input2)?;
    let (input2, _) = le_u32::<_, Error<_>>(input2)?;

    let (input2, currentScrollX) = le_f32::<_, Error<_>>(input2)?;
    let (input2, currentScrollY) = le_f32::<_, Error<_>>(input2)?;

    let (input2, _) = le_u32::<_, Error<_>>(input2)?;
    let (input2, _) = le_u32::<_, Error<_>>(input2)?;
    let (input2, _) = le_u32::<_, Error<_>>(input2)?;
    let (input2, _) = le_u32::<_, Error<_>>(input2)?;

    let (input2, blendIndex) = le_u32::<_, Error<_>>(input2)?;

    Ok((input2, VisualMaterialTexture {
        offset: offset as u32,
        off_texture: off_texture,
        textureOp: textureOp,
        shadingMode: shadingMode,
        uvFunction: uvFunction,
        scrollByte: scrollByte,
        properties: properties,
        scrollX: scrollX,
        scrollY: scrollY,
        rotateSpeed: rotateSpeed,
        rotateDirection: rotateDirection,
        currentScrollX: currentScrollX,
        currentScrollY: currentScrollY,
        blendIndex: blendIndex
    }))
}

fn object_type_list<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], Vec<ObjectType>> {
    let (input, off_names_first) = le_u32::<_, Error<_>>(input)?;
    let (input, off_names_last) = le_u32::<_, Error<_>>(input)?;
    let (input, num_names) = le_u32::<_, Error<_>>(input)?;

    let object_type_off = &buffer[(off_names_first as usize + 4)..];
    let (foo, obj) = count(|x| object_type(x, buffer), num_names as usize)(object_type_off)?;

    Ok((input, obj))
}

fn object_type<'a>(input: &'a[u8], buffer: &'a[u8]) -> IResult<&'a[u8], ObjectType> {
    let (input, off_names_next) = le_u32::<_, Error<_>>(input)?;
    let (input, off_names_prev) = le_u32::<_, Error<_>>(input)?;
    let (input, off_header) = le_u32::<_, Error<_>>(input)?;
    let (input, off_name) = le_u32::<_, Error<_>>(input)?;

    let (input, unk1) = le_u8::<_, Error<_>>(input)?;
    let (input, id) = le_u8::<_, Error<_>>(input)?;
    let (input, unk2) = le_u16::<_, Error<_>>(input)?;

    let name = &buffer[(off_name as usize + 4)..];

    let (done, name) = map(take_while(|c| c != 0), |cs: &[u8]| String::from_utf8_lossy(cs).into_owned())(name)?;

    let next = &buffer[(off_names_next as usize + 4)..];

    Ok((next, ObjectType {
        name: name,
        unk1: unk1,
        id: id,
        unk2: unk2
    }))
}

//could move the texture stuff into its own function just taking buffer
//and handle the offset read and no texture case elsewhere
fn texture<'a>(input: &'a[u8], buffer: &'a[u8], buffers: &'a[&Vec<u8>; 2], pointers: &HashMap<usize, usize>) -> IResult<&'a[u8], Option<TextureInfo>> {
    let inptr = input.as_ptr() as usize;
    let buffPtr = buffer.as_ptr() as usize;
    let dataStreamOffset = inptr - buffPtr - 4;
    let (input, off_texture) = le_u32(input)?;
    if off_texture == 0 { return Ok((input, None)); }

    let fileId = *pointers.get(&dataStreamOffset).unwrap_or(&1);  //can't hardcode this as default file id really
    let texture = &buffers[fileId][(off_texture as usize + 4)..];
    let (texture, field0) = le_u32(texture)?;
    let (texture, field4) = le_u16(texture)?;
    let (texture, field6) = le_u16(texture)?;
    let (texture, off_tempBuffer) = le_u32(texture)?;
    let (texture, fieldC) = le_u32(texture)?;
    let (texture, field10) = le_u32(texture)?;
    let (texture, flags) = le_u32(texture)?;
    let (texture, height_) = le_u16(texture)?;
    let (texture, width_) = le_u16(texture)?;
    let (texture, height) = le_u16(texture)?;
    let (texture, width) = le_u16(texture)?;
    let (texture, currentScrollX) = le_u32(texture)?;
    let (texture, currentScrollY) = le_u32(texture)?;
    let (texture, textureScrollingEnabled) = le_u32(texture)?;
    let (texture, alphaMask) = le_u32(texture)?;
    let (texture, field30) = le_u32(texture)?;
    let (texture, numMipmaps) = le_u32(texture)?;
    let (texture, field38) = le_u32(texture)?;
    let (texture, field3C) = le_u32(texture)?;
    let (texture, field40) = le_u32(texture)?;
    let (texture, field44) = le_u32(texture)?;
    let (texture, field48) = le_u8(texture)?;
    let (texture, flagsByte) = le_u8(texture)?;
    let (_, name) = map(take_while(|c| c != 0), |cs: &[u8]| String::from_utf8_lossy(cs).into_owned())(texture)?;

    //need to save file ID as well, to serialize back to files correctly
    Ok((input, Some(TextureInfo {
        field0: field0,
        field4: field4,
        field6: field6,
        off_tempBuffer: off_tempBuffer,
        fieldC: fieldC,
        field10: field10,
        flags: flags,
        height_ : height_,
        width_: width_,
        height: height,
        width: width,
        currentScrollX: currentScrollX,
        currentScrollY: currentScrollY,
        textureScrollingEnabled: textureScrollingEnabled,
        alphaMask: alphaMask,
        field30: field30,
        numMipmaps: numMipmaps,
        field38: field38,
        field3C: field3C,
        field40: field40,
        field44: field44,
        field48: field48,
        flagsByte: flagsByte,
        name: name
    })))
}