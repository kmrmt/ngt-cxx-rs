use chan::chan_select;

#[cxx::bridge]
mod ffi {
    #[repr(i32)]
    enum ObjectType {
        None = 0,
        Uint8 = 1,
        Float = 2,
        Float16 = 3,
    }

    #[repr(i32)]
    enum DistanceType {
        None = -1,
        L1 = 0,
        L2 = 1,
        Hamming = 2,
        Angle = 3,
        Cosine = 4,
        NormalizedAngle = 5,
        NormalizedCosine = 6,
        Jaccard = 7,
        SparseJaccard = 8,
        NormalizedL2 = 9,
        Poincare = 100,
        Lorentz = 101,
    }

    unsafe extern "C++" {
        include!("ngt-cxx-rs/src/helper.hpp");

        type Property;
        fn new_property() -> UniquePtr<Property>;
        fn set_dimension(self: Pin<&mut Property>, dimension: i32);
        fn set_object_type(self: Pin<&mut Property>, t: ObjectType);
        fn set_distance_type(self: Pin<&mut Property>, t: DistanceType);

        type Index;
        fn new_index(p: Pin<&mut Property>) -> UniquePtr<Index>;
        fn insert(self: Pin<&mut Index>, v: &Vec<f32>) -> u32;
        fn create_index(self: Pin<&mut Index>, pool_size: u32);
        fn remove(self: Pin<&mut Index>, id: u32);
    }
}

fn stat(op: &str) {
    let me = procfs::process::Process::myself().unwrap();
    let st = me.status().unwrap();
    println!(
        "{}\t{}\t{}\t{}\t{}\t{}",
        chrono::Local::now(),
        op,
        st.vmpeak.unwrap(),
        st.vmsize.unwrap(),
        st.vmhwm.unwrap(),
        st.vmrss.unwrap()
    );
}
fn main() {
    println!("time\toperation\tVmPeak\tVmSize\tVmHWM\tVmRSS");
    stat("start");
    let f = hdf5::File::open("sift-128-euclidean.hdf5").unwrap();
    let d = f.dataset("train").unwrap();
    let vecs: ndarray::Array2<f32> = d.read_2d::<f32>().unwrap();
    let shape = vecs.shape();
    f.close();
    stat("load");

    let mut p = ffi::new_property();
    p.pin_mut().set_dimension(shape[1] as i32);
    p.pin_mut().set_distance_type(ffi::DistanceType::L2);
    p.pin_mut().set_object_type(ffi::ObjectType::Float);

    let mut index = ffi::new_index(p.pin_mut());

    let tick = chan::after(std::time::Duration::from_secs(3 * 60 * 60));
    loop {
        chan_select! {
            default => {
                stat("init");
                let ids: Vec<_> = vecs
                    .outer_iter()
                    .map(|v| index.pin_mut().insert(&v.to_vec()))
                    .collect();
                stat("insert");
                index.pin_mut().create_index(8);
                stat("createindex");
                ids.iter().for_each(|id| index.pin_mut().remove(*id));
                stat("remove");
            },
            tick.recv() => {
                stat("finish");
                return;
            }
        }
    }
}
