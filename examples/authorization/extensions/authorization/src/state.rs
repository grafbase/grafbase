#[derive(Default, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
pub struct State {
    pub denied_ids: Vec<DeniedIds>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Debug)]
pub struct DeniedIds {
    pub query_element_id: u32,
    pub authorized_ids: Vec<u32>,
}
