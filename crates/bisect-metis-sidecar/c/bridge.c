/* ABI v1. Source pinned by the builder; no user code or path is executed. */
#include <metis.h>
_Static_assert(sizeof(idx_t)==4 && sizeof(real_t)==4,"BISECT requires idx32/real32");
_Static_assert(METIS_NOPTIONS==40 && METIS_OPTION_NITER==7 && METIS_OPTION_SEED==9
    && METIS_OPTION_MINCONN==11 && METIS_OPTION_CONTIG==12 && METIS_OPTION_UFACTOR==17,
    "METIS options layout changed");
int bisect_metis_partition(int mode,idx_t n,idx_t ncon,idx_t k,idx_t *xadj,
    idx_t *adjncy,idx_t *vwgt,idx_t *adjwgt,real_t *tpwgts,real_t *ubvec,
    idx_t *options,idx_t *cut,idx_t *part) {
    if(mode==0)return METIS_PartGraphKway(&n,&ncon,xadj,adjncy,vwgt,0,adjwgt,
        &k,tpwgts,ubvec,options,cut,part);
    if(mode==1)return METIS_PartGraphRecursive(&n,&ncon,xadj,adjncy,vwgt,0,adjwgt,
        &k,tpwgts,ubvec,options,cut,part);
    return METIS_ERROR_INPUT;
}
