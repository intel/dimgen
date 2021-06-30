use argh::FromArgs;

#[derive(FromArgs)]
/// generate source container images form binary containers
pub struct Args {
    /// name of the image for which source image has to be generated
    #[argh(option, short = 'i')]
    pub image: String,

    /// generate sources for base image and additional layers seperately
    #[argh(switch, short = 's')]
    pub seperate: bool,
}
