use libc::{c_void, user_regs_struct, PT_NULL}; //Nos facilita poder utilizar codigo parecido a C en Rut
use nix::sys::ptrace::*; //Libreria Nix es dependiente del sistema, resultados pueden cambiar y nos permite seguir un proceso.
use nix::sys::wait::waitpid; //Libreria Nix es dependiente del sistema, resultados pueden cambiar y nos permite esperar por el processid.
use nix::sys::ptrace; //Libreria Nix es dependiente del sistema, resultados pueden cambiar.
use std::os::unix::process::CommandExt;//Libreria Estandar de rust.
use std::process::Command;//Libreria Estandar de rust.
use std::ptr;//Libreria Estandar de rust.
use std::collections::HashMap;//Libreria Estandar de rust para obtener HashMaps.
use std::mem;//Libreria Estandar de rust para el manejo de memoria.


mod llamadas_sistema; //Llamamos a la colleccion de nombre de llamadas de sistema.
#[allow(dead_code)]
//Esta funcion nos permite seguir el proceso hijo utilizando ptrace
fn traceme() -> std::io::Result<()> {
    match ptrace::traceme() {
        Ok(()) => Ok(()),
        Err(::nix::Error::Sys(errno)) => Err(std::io::Error::from_raw_os_error(errno as i32)),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, e)),
    }
}
#[allow(dead_code)]
#[allow(deprecated)]
//Esta funcion nos permite obtener los registros del proceso utilizando ptrace
pub fn get_regs(id_proceso: nix::unistd::Pid) -> Result<user_regs_struct, nix::Error> {
    unsafe { //Tenemos que desabilitar la guarantia de inmutabilidad del compilador
        let mut registros: user_regs_struct = mem::uninitialized();

        //nos permite eliminar el warning del codigo incluso si es depreciado
        let res = ptrace::ptrace(
            Request::PTRACE_GETREGS,
            id_proceso,
            PT_NULL as *mut c_void,
            &mut registros as *mut _ as *mut c_void,
        );
        res.map(|_| registros)
    }
}
#[allow(deprecated)]
fn main() {
    let argv: Vec<_> = std::env::args().collect();
    let mut terminal = Command::new(&argv[1]);
    for arg in argv {
        println!("{}", arg);
        terminal.arg(arg);
    }
    //Se guarda la cantidad de llamadas.
    let mut map = HashMap::new();

    //Permite el proceso hijo se seguido
    terminal.before_exec(traceme);
    let hijo = terminal.spawn().expect("Fallo el proceso hijo");
    let id_proceso = nix::unistd::Pid::from_raw(hijo.id() as libc::pid_t);

    //Permite al proceso padre ser detenido, utilizando SIGTRAP.
    ptrace::setoptions(
        id_proceso,
        Options::PTRACE_O_TRACESYSGOOD | Options::PTRACE_O_TRACEEXEC,
    )
    .unwrap(); //Nos permite verifica que no retorne un error, en caso de retornarlo hace panic.
    waitpid(id_proceso, None).ok();//Esperamos a que el proceso termine, en caso de error lo convertimos en none.


    //Detenemos el ptrace cuando entramos y salimos de una llamada de sistema.
    let mut salida = true;
    loop {
        //obtiene los registros de la direccion donde ptrace se detuvo.
        let registros = match get_regs(id_proceso) {
            Ok(x) => x,
            Err(err) => {
                eprintln!("Final del ptrace {:?}", err);
                break;
            }
        };
        if salida {
            //Lee los datos guardados en NOMBRES_SYSCALL y los compara, para obtener el nombre
            //del Error con el numero para desplegarlo correctamente.  
            let nombres_syscall = llamadas_sistema::NOMBRES_SYSCALL[(registros.orig_rax) as usize];

            match map.get(&nombres_syscall) {
                Some(&numero) => map.insert(nombres_syscall, numero + 1),
                _ => map.insert(nombres_syscall, 1),
            };
        }
        unsafe { //Tenemos que desabilitar la guarantia de inmutabilidad del compilador
            ptrace(
                Request::PTRACE_SYSCALL,
                id_proceso,
                ptr::null_mut(),
                ptr::null_mut(),
            ).ok(); //Nos permite manejar en caso de error lo convertimsos en None.
        }

        waitpid(id_proceso, None).ok(); //Esperamos a que el proceso termine, en caso de error lo convertimos en none.
        salida = !salida;
    }
    for (syscall, &numero) in map.iter() {
        println!("{}: {}", syscall, numero);
    }
}
