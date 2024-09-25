use build_info_build::DependencyDepth;

fn main() {
    build_info_build::build_script().collect_runtime_dependencies(DependencyDepth::Depth(1));
}
