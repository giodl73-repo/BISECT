/* Feasibility fixture, not the production engine. Same vendored C METIS API. */
#include <metis.h>
#include <stdlib.h>

void bisect_rng_seed(unsigned int seed) { srand(seed); }
int bisect_rng_next(void) { return rand(); }

/* Same options as the Rust native matrix; caller owns validated CSR arrays. */
int bisect_metis_case(idx_t n, idx_t ncon, idx_t k, idx_t *xadj,
    idx_t *adjncy, idx_t *vwgt, idx_t *adjwgt, real_t *tpwgts,
    idx_t seed, idx_t *cut, idx_t *part) {
    idx_t options[METIS_NOPTIONS];
    METIS_SetDefaultOptions(options);
    options[METIS_OPTION_SEED]=seed;
    options[METIS_OPTION_UFACTOR]=30;
    options[METIS_OPTION_NITER]=10;
    options[METIS_OPTION_CONTIG]=1;
    options[METIS_OPTION_MINCONN]=1;
    return METIS_PartGraphKway(&n,&ncon,xadj,adjncy,vwgt,0,adjwgt,&k,
        tpwgts,0,options,cut,part);
}

int bisect_metis_probe(void) {
    idx_t n=4,ncon=1,k=2,xadj[]={0,2,4,6,8},adjncy[]={1,3,0,2,1,3,0,2};
    idx_t options[METIS_NOPTIONS],cut=0,part[4]={0};
    METIS_SetDefaultOptions(options); options[METIS_OPTION_SEED]=7;
    int code=METIS_PartGraphKway(&n,&ncon,xadj,adjncy,0,0,0,&k,0,0,options,&cut,part);
    if(code!=METIS_OK)return code;
    return (cut<<8)|part[0]|(part[1]<<1)|(part[2]<<2)|(part[3]<<3);
}
int bisect_metis_error_probe(void) {
    idx_t n=2,ncon=1,k=2,xadj[]={0,0,0},adjncy[]={0},part[2]={0},options[METIS_NOPTIONS],cut=0;
    METIS_SetDefaultOptions(options); options[METIS_OPTION_CONTIG]=1;
    return METIS_PartGraphKway(&n,&ncon,xadj,adjncy,0,0,0,&k,0,0,options,&cut,part);
}
int bisect_metis_invalid_option_probe(void) {
    idx_t n=4,ncon=1,k=2,xadj[]={0,2,4,6,8},adjncy[]={1,3,0,2,1,3,0,2};
    idx_t options[METIS_NOPTIONS],cut=0,part[4]={0};
    METIS_SetDefaultOptions(options); options[METIS_OPTION_NITER]=0;
    return METIS_PartGraphKway(&n,&ncon,xadj,adjncy,0,0,0,&k,0,0,options,&cut,part);
}
