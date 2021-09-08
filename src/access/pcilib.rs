
extern "C" {
    fn pci_lookup_method(name: &str) -> i32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        //{
        //
        //    pacc = pci_alloc();
        //    pacc->error = die;
        //    pci_filter_init(pacc, &filter);
        //
        //
        //    pci_init(pacc);
        //    if (opt_map_mode)
        //    {
        //        if (need_topology)
        //            die("Bus mapping mode does not recognize bus topology");
        //        map_the_bus();
        //    }
        //    else
        //    {
        //        scan_devices();
        //        sort_them();
        //        if (need_topology)
        //            grow_tree();
        //        if (opt_tree)
        //            show_forest(opt_filter ? &filter : NULL);
        //        else
        //            show();
        //    }
        //    show_kernel_cleanup();
        //    pci_cleanup(pacc);
        //
        //}
        let is_found = unsafe { pci_lookup_method("linux-sysfs\x00") };
        dbg!(is_found);
    }
}
