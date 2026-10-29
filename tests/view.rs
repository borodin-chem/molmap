use molmap::{
    atomic::{AtomMap0, AtomMap2},
    entities::{Atom, Selection::All},
    *,
};

#[test]
fn views_usage() {
    let mut mm = AtomMap0::new();
    let h1 = mm.add_atom(Element::H);
    let _atom_views = mm.views::<Atom>(All).unwrap();
    let _atom_views = mm.views([h1]).unwrap();
}

#[test]
fn all_maps_atom_data() {
    // Tests that views of atoms in all maps provide access to the basic data
    // i.e. that the generic view works properly
    let mut am0 = AtomMap0::new();
    let h0 = am0.add_atom(Element::H);
    assert_eq!(am0.view(h0).unwrap().element(), Element::H);
    let mut am2 = AtomMap2::new();
    let h2 = am2.add_atom(Element::H, Point2::new(1.0, 2.5));
    assert_eq!(am2.view(h2).unwrap().element(), Element::H);
    //let mut am3 = AtomMap3::new();
    //let h3 = am3.add_atom(Element::H);
}
