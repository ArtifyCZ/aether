#[derive(Debug)]
pub struct KernelArgs<'cmdline> {
    init_program: &'cmdline str,
}

impl<'cmdline> KernelArgs<'cmdline> {
    pub fn parse(cmdline: &'cmdline str) -> Self {
        // Default values
        let mut args = Self {
            init_program: "/bin/init",
        };

        for entry in cmdline.split_whitespace() {
            if let Some((key, value)) = entry.split_once('=') {
                match key {
                    "init" => args.init_program = value,
                    _ => {}, // ignore unknown
                }
            }
        }

        args
    }
}
