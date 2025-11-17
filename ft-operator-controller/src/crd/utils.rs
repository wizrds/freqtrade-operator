use kube::CustomResourceExt;
use kube::core::crd::merge_crds;

use crate::crd::v1alpha1::bot::Bot as V1Alpha1Bot;


/// Generate the CRDs for the operator
pub fn generate_crds() {
    for crd in vec![
        merge_crds(vec![V1Alpha1Bot::crd()], "v1alpha1").expect("failed to merge Bot CRDs"),
    ] {
        println!("---");
        println!("{}", serde_norway::to_string(&crd).unwrap());
    }
}
