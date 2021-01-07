use super::*;

/// An encoder for the instance section.
///
/// Note that this is part of the [module linking proposal][proposal] and is not
/// currently part of stable WebAssembly.
///
/// [proposal]: https://github.com/webassembly/module-linking
///
/// # Example
///
/// ```
/// use wasm_encoder::{Module, InstanceSection, Export};
///
/// let mut instances = InstanceSection::new();
/// instances.instantiate(0, vec![
///     ("x", None, Export::Function(0)),
///     ("", Some("y"), Export::Module(2)),
///     ("foo", None, Export::Global(0)),
/// ]);
///
/// let mut module = Module::new();
/// module.section(&instances);
///
/// let wasm_bytes = module.finish();
/// ```
pub struct InstanceSection {
    bytes: Vec<u8>,
    num_added: u32,
}

impl InstanceSection {
    /// Construct a new instance section encoder.
    pub fn new() -> InstanceSection {
        InstanceSection {
            bytes: vec![],
            num_added: 0,
        }
    }

    /// Define an instantiation of the given module with the given items as
    /// arguments to the instantiation.
    pub fn instantiate<'a, I>(&mut self, module: u32, args: I) -> &mut Self
    where
        I: IntoIterator<Item = (&'a str, Option<&'a str>, Export)>,
        I::IntoIter: ExactSizeIterator,
    {
        let args = args.into_iter();

        self.bytes.push(0x00);
        self.bytes.extend(encoders::u32(module));
        self.bytes
            .extend(encoders::u32(u32::try_from(args.len()).unwrap()));
        for (name, field, export) in args {
            self.bytes.extend(encoders::str(name));
            match field {
                Some(field) => {
                    self.bytes.push(0x01);
                    self.bytes.extend(encoders::str(field));
                }
                None => {
                    self.bytes.push(0x00);
                }
            }
            export.encode(&mut self.bytes);
        }
        self.num_added += 1;
        self
    }
}

impl Section for InstanceSection {
    fn id(&self) -> u8 {
        SectionId::Instance.into()
    }

    fn encode<S>(&self, sink: &mut S)
    where
        S: Extend<u8>,
    {
        let num_added = encoders::u32(self.num_added);
        let n = num_added.len();
        sink.extend(
            encoders::u32(u32::try_from(n + self.bytes.len()).unwrap())
                .chain(num_added)
                .chain(self.bytes.iter().copied()),
        );
    }
}
