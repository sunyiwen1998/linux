struct kvm_arch_memory_slot {
    rmap         :     &[struct kvm_rmap_head, KVM_NR_PAGE_SIZES] ,
    lpage_info   :     &[struct kvm_lpage_info, KVM_NR_PAGE_SIZES - 1] ,
    gfn_track    :     &[unsigned short, KVM_PAGE_TRACK_MAX]
};

struct kvm_rmap_head {
    val : unsigned long 
};

struct kvm_lpage_info {
    disallow_lpage : int 
};

struct kvm_io_bus {
    dev_count : int,
    ioeventfd_count : int ,
    range : Vec<struct kvm_io_range> 
};

struct kvm_io_range {
    addr : gpa_t,                        // typedef u64            gpa_t;
    len :  int ,
    dev :  &struct kvm_io_device ;
};