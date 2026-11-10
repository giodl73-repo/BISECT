// Preserve the actual native raw filter vector in its command review. The
// manifest adapter accepts two-letter codes; an unknown marker keeps a nonempty
// wholly unmatchable filter from accidentally becoming the 'all states' filter.
export function nativeRunStateFilters(review){
 const values=[...new Set(review.states.map(s=>s.toUpperCase()).filter(s=>/^[A-Z]{2}$/.test(s)))];
 return review.states.length&&!values.length?['ZZ']:values;
}
