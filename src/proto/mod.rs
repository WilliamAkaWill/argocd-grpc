pub mod application {
    #![allow(dead_code, unused_imports, non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/application.rs"));
}
pub mod repository {
    #![allow(dead_code, unused_imports, non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/repository.rs"));
}
pub mod events {
    #![allow(dead_code, unused_imports, non_camel_case_types)]
    include!(concat!(env!("OUT_DIR"), "/events.rs"));
}
pub mod github {
    pub mod com {
        pub mod argoproj {
            pub mod argo_cd {
                pub mod v3 {
                    pub mod pkg {
                        pub mod apis {
                            pub mod application {
                                pub mod v1alpha1 {
                                    #![allow(dead_code, unused_imports, non_camel_case_types)]
                                    include!(concat!(
                                        env!("OUT_DIR"),
                                        "/github.com.argoproj.argo_cd.v3.pkg.apis.application.v1alpha1.rs"
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
pub mod google {
    pub mod api {
        #![allow(dead_code, unused_imports, non_camel_case_types)]
        include!(concat!(env!("OUT_DIR"), "/google.api.rs"));
    }
    pub mod protobuf {
        #![allow(dead_code, unused_imports, non_camel_case_types)]
        include!(concat!(env!("OUT_DIR"), "/google.protobuf.rs"));
    }
}
pub mod k8s {
    pub mod io {
        pub mod api {
            pub mod core {
                pub mod v1 {
                    #![allow(dead_code, unused_imports, non_camel_case_types)]
                    include!(concat!(env!("OUT_DIR"), "/k8s.io.api.core.v1.rs"));
                }
            }
        }
        pub mod apiextensions_apiserver {
            pub mod pkg {
                pub mod apis {
                    pub mod apiextensions {
                        pub mod v1 {
                            #![allow(dead_code, unused_imports, non_camel_case_types)]
                            include!(concat!(env!("OUT_DIR"), "/k8s.io.apiextensions_apiserver.pkg.apis.apiextensions.v1.rs"));
                        }
                    }
                }
            }
        }
        pub mod apimachinery {
            pub mod pkg {
                pub mod api {
                    pub mod resource {
                        #![allow(dead_code, unused_imports, non_camel_case_types)]
                        include!(concat!(env!("OUT_DIR"), "/k8s.io.apimachinery.pkg.api.resource.rs"));
                    }
                }
                pub mod apis {
                    pub mod meta {
                        pub mod v1 {
                            #![allow(dead_code, unused_imports, non_camel_case_types)]
                            include!(concat!(env!("OUT_DIR"), "/k8s.io.apimachinery.pkg.apis.meta.v1.rs"));
                        }
                    }
                }
                pub mod runtime {
                    #![allow(dead_code, unused_imports, non_camel_case_types)]
                    include!(concat!(env!("OUT_DIR"), "/k8s.io.apimachinery.pkg.runtime.rs"));
                }
                pub mod util {
                    pub mod intstr {
                        #![allow(dead_code, unused_imports, non_camel_case_types)]
                        include!(concat!(env!("OUT_DIR"), "/k8s.io.apimachinery.pkg.util.intstr.rs"));
                    }
                }
            }
        }
    }
}