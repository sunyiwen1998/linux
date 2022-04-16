struct kvm_io_device {
    //const struct kvm_io_device_ops *ops;
    const ops : &struct kvm_io_device_ops
};