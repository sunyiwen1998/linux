// This struct is not fully remodeled
static struct kvm *kvm_create_vm(unsigned long type)
{
  struct kvm *kvm = kvm_arch_alloc_vm();
  struct kvm_memslots *slots;
  int r = -ENOMEM;
  int i, j;

  if (!kvm)
                return ERR_PTR(-ENOMEM);

        KVM_MMU_LOCK_INIT(kvm);
        mmgrab(current->mm);
        kvm->mm = current->mm;
        kvm_eventfd_init(kvm);
        mutex_init(&kvm->lock);
        mutex_init(&kvm->irq_lock);
        mutex_init(&kvm->slots_lock);
        mutex_init(&kvm->slots_arch_lock);
        spin_lock_init(&kvm->mn_invalidate_lock);
        rcuwait_init(&kvm->mn_memslots_update_rcuwait);
        xa_init(&kvm->vcpu_array);

        INIT_LIST_HEAD(&kvm->gpc_list);
        spin_lock_init(&kvm->gpc_lock);

        INIT_LIST_HEAD(&kvm->devices);

        BUILD_BUG_ON(KVM_MEM_SLOTS_NUM > SHRT_MAX);

        if (init_srcu_struct(&kvm->srcu))
                goto out_err_no_srcu;
        if (init_srcu_struct(&kvm->irq_srcu))
                goto out_err_no_irq_srcu;

        refcount_set(&kvm->users_count, 1);
        for (i = 0; i < KVM_ADDRESS_SPACE_NUM; i++) {
                for (j = 0; j < 2; j++) {
                        slots = &kvm->__memslots[i][j];

                        atomic_long_set(&slots->last_used_slot, (unsigned long)NULL);
                        slots->hva_tree = RB_ROOT_CACHED;
                        slots->gfn_tree = RB_ROOT;
                        hash_init(slots->id_hash);
                        slots->node_idx = j;

                        /* Generations must be different for each address space. */
                        slots->generation = i;
                }

                rcu_assign_pointer(kvm->memslots[i], &kvm->__memslots[i][0]);
        }

        for (i = 0; i < KVM_NR_BUSES; i++) {
                rcu_assign_pointer(kvm->buses[i],
                        kzalloc(sizeof(struct kvm_io_bus), GFP_KERNEL_ACCOUNT));
                if (!kvm->buses[i])
                        goto out_err_no_arch_destroy_vm;
        }

        kvm->max_halt_poll_ns = halt_poll_ns;

        r = kvm_arch_init_vm(kvm, type);
        if (r)
                goto out_err_no_arch_destroy_vm;

        r = hardware_enable_all();
        if (r)
                goto out_err_no_disable;

#ifdef CONFIG_HAVE_KVM_IRQFD
        INIT_HLIST_HEAD(&kvm->irq_ack_notifier_list);
#endif

        r = kvm_init_mmu_notifier(kvm);
        if (r)
                goto out_err_no_mmu_notifier;

        r = kvm_arch_post_init_vm(kvm);
        if (r)
                goto out_err;

        mutex_lock(&kvm_lock);
        list_add(&kvm->vm_list, &vm_list);
        mutex_unlock(&kvm_lock);

        preempt_notifier_inc();
        kvm_init_pm_notifier(kvm);

        return kvm;

out_err:
#if defined(CONFIG_MMU_NOTIFIER) && defined(KVM_ARCH_WANT_MMU_NOTIFIER)
        if (kvm->mmu_notifier.ops)
                mmu_notifier_unregister(&kvm->mmu_notifier, current->mm);
#endif
out_err_no_mmu_notifier:
        hardware_disable_all();
out_err_no_disable:
        kvm_arch_destroy_vm(kvm);
out_err_no_arch_destroy_vm:
        WARN_ON_ONCE(!refcount_dec_and_test(&kvm->users_count));
        for (i = 0; i < KVM_NR_BUSES; i++)
                kfree(kvm_get_bus(kvm, i));
        cleanup_srcu_struct(&kvm->irq_srcu);
out_err_no_irq_srcu:
        cleanup_srcu_struct(&kvm->srcu);
out_err_no_srcu:
        kvm_arch_free_vm(kvm);
        mmdrop(current->mm);
        return ERR_PTR(r);
}