//! The subset of metis 0.2's Graph interface used by BISECT. C executes in a
//! separate WASM memory. This module never substitutes another algorithm.
use std::cell::Cell;
use std::fmt;

thread_local! { static ENABLED: Cell<bool> = const { Cell::new(false) }; }
pub fn is_enabled() -> bool { ENABLED.with(Cell::get) }
pub struct BackendGuard(bool);
impl BackendGuard {
    pub fn enter(enabled: bool) -> Self { Self(ENABLED.with(|cell| cell.replace(enabled))) }
}
impl Drop for BackendGuard {
    fn drop(&mut self) { ENABLED.with(|cell| cell.set(self.0)); }
}

#[derive(Debug,PartialEq,Eq)]
pub enum Error { Input, Memory, Other }
impl fmt::Display for Error {
    fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result {
        f.write_str(match self {Self::Input=>"invalid input",Self::Memory=>"out of memory",Self::Other=>"METIS returned an error"})
    }
}
impl std::error::Error for Error {}

pub mod option {
    // Pinned metis-sys 0.3.2 header; bridge.c checks these at C compile time.
    pub trait Opt { fn pair(self)->(usize,i32); }
    macro_rules! integer { ($name:ident,$index:expr)=>{
        pub struct $name(pub i32);
        impl Opt for $name {fn pair(self)->(usize,i32){($index,self.0)}}
    }; }
    macro_rules! boolean { ($name:ident,$index:expr)=>{
        pub struct $name(pub bool);
        impl Opt for $name {fn pair(self)->(usize,i32){($index,i32::from(self.0))}}
    }; }
    integer!(NIter,7); integer!(Seed,9); integer!(UFactor,17);
    boolean!(MinConn,11); boolean!(Contig,12);
}

pub struct Graph<'a> {
    ncon:i32,k:i32,xadj:&'a[i32],adjncy:&'a[i32],
    vwgt:Option<&'a[i32]>,adjwgt:Option<&'a[i32]>,
    tpwgts:Option<&'a[f32]>,ubvec:Option<&'a[f32]>,options:[i32;40],
}
impl<'a> Graph<'a> {
    pub fn new(ncon:i32,k:i32,xadj:&'a[i32],adjncy:&'a[i32])->Result<Self,Error>{
        if ncon<=0||k<=0||xadj.is_empty()||xadj.len()>i32::MAX as usize||adjncy.len()>i32::MAX as usize
            ||xadj[0]!=0||xadj.last().copied()!=Some(adjncy.len() as i32)
            ||xadj.windows(2).any(|pair|pair[0]>pair[1])
            ||adjncy.iter().any(|&v|v<0||v as usize>=xadj.len()-1) {return Err(Error::Input);}
        Ok(Self{ncon,k,xadj,adjncy,vwgt:None,adjwgt:None,tpwgts:None,ubvec:None,options:[-1;40]})
    }
    pub fn set_vwgt(mut self,value:&'a[i32])->Self {self.vwgt=Some(value);self}
    pub fn set_adjwgt(mut self,value:&'a[i32])->Self {self.adjwgt=Some(value);self}
    pub fn set_tpwgts(mut self,value:&'a[f32])->Self {self.tpwgts=Some(value);self}
    pub fn set_ubvec(mut self,value:&'a[f32])->Self {self.ubvec=Some(value);self}
    pub fn set_option<O:option::Opt>(mut self,value:O)->Self {let (index,value)=value.pair();self.options[index]=value;self}
    pub fn part_kway(self,part:&mut[i32])->Result<i32,Error>{self.partition(part,0)}
    pub fn part_recursive(self,part:&mut[i32])->Result<i32,Error>{self.partition(part,1)}
    fn partition(self,part:&mut[i32],mode:i32)->Result<i32,Error>{
        let n=self.xadj.len()-1;
        let nweights=n.checked_mul(self.ncon as usize).ok_or(Error::Input)?;
        let ntargets=(self.k as usize).checked_mul(self.ncon as usize).ok_or(Error::Input)?;
        if n==0||part.len()!=n
            ||self.vwgt.is_some_and(|v|v.len()!=nweights||v.iter().any(|&w|w<0))
            ||self.adjwgt.is_some_and(|v|v.len()!=self.adjncy.len()||v.iter().any(|&w|w<=0))
            ||self.tpwgts.is_some_and(|v|v.len()!=ntargets||v.iter().any(|w|!w.is_finite()||*w<=0.))
            ||self.ubvec.is_some_and(|v|v.len()!=self.ncon as usize||v.iter().any(|w|!w.is_finite()||*w<=1.))
        {return Err(Error::Input);}
        let mut cut=0;
        #[cfg(target_arch="wasm32")]
        let code=unsafe {bisect_metis_partition(mode,n as i32,self.ncon,self.k,
            self.xadj.as_ptr(),self.adjncy.as_ptr(),
            self.vwgt.map_or(std::ptr::null(),|v|v.as_ptr()),
            self.adjwgt.map_or(std::ptr::null(),|v|v.as_ptr()),
            self.tpwgts.map_or(std::ptr::null(),|v|v.as_ptr()),
            self.ubvec.map_or(std::ptr::null(),|v|v.as_ptr()),
            self.options.as_ptr(),&mut cut,part.as_mut_ptr())};
        #[cfg(not(target_arch="wasm32"))]
        let code={let _=(&self,mode,&mut cut);-4};
        match code {
            1 if cut>=0&&part.iter().all(|&p|p>=0&&p<self.k)=>Ok(cut),
            -2=>Err(Error::Input),-3=>Err(Error::Memory),_=>Err(Error::Other)
        }
    }
}
#[cfg(target_arch="wasm32")]
extern "C" {
    fn bisect_metis_partition(mode:i32,n:i32,ncon:i32,k:i32,xadj:*const i32,
        adjncy:*const i32,vwgt:*const i32,adjwgt:*const i32,tpwgts:*const f32,
        ubvec:*const f32,options:*const i32,cut:*mut i32,part:*mut i32)->i32;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rejects_invalid_csr(){
        for (x,a) in [(vec![],vec![]),(vec![1,1],vec![0]),(vec![0,2,1],vec![0]),(vec![0,1],vec![1]),(vec![0,-1],vec![])] {
            assert!(matches!(Graph::new(1,2,&x,&a),Err(Error::Input)));
        }
    }
    #[test] fn dimensions_fail_before_host_call(){
        let graph=||Graph::new(1,2,&[0,1,2],&[1,0]).unwrap();
        assert_eq!(graph().set_vwgt(&[1]).part_kway(&mut[0;2]),Err(Error::Input));
        assert_eq!(graph().set_adjwgt(&[1,0]).part_kway(&mut[0;2]),Err(Error::Input));
        assert_eq!(graph().set_tpwgts(&[f32::NAN,0.5]).part_kway(&mut[0;2]),Err(Error::Input));
        assert_eq!(graph().set_ubvec(&[1.]).part_kway(&mut[0;2]),Err(Error::Input));
        assert_eq!(graph().part_kway(&mut[0;1]),Err(Error::Input));
    }
    #[test] fn nested_selection_restores(){
        assert!(!is_enabled());
        {let _outer=BackendGuard::enter(true);assert!(is_enabled());
            {let _inner=BackendGuard::enter(false);assert!(!is_enabled());}
            assert!(is_enabled());}
        assert!(!is_enabled());
    }
}
