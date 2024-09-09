#[derive(raad_codegen::Read, raad_codegen::Write)]
struct X {
    y: u8,
}
#[derive(raad_codegen::Write, raad_codegen::Read)]
struct Z(X);
