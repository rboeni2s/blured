use blured::{service, service::application::Application};
use keep::Guard;
use plug::logger;


fn main()
{
    if let Err(e) = logger::init()
    {
        eprintln!("Error: Failed to initialize logger: {e}");
    }

    match service::build_reg()
    {
        Ok(reg) =>
        {
            let reg = Guard::new(reg);
            if let Err(e) = reg.get_unchecked::<Application>().run(reg.clone())
            {
                log::error!("Fatal error: {e}");
            }
        }

        Err(e) => log::error!("Failed to build a service registry, quitting because of: {e}"),
    }
}
