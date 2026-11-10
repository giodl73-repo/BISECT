"""Checked comparison for the declared four-unit native CLI counterfactuals.

Original files are never changed. Native verify --dry-run must additionally
check the typed recipe digest before these comparison values are used.
"""
import copy, hashlib, json

def canonical_hash(value):
    return 'sha256:'+hashlib.sha256(json.dumps(value,sort_keys=True,separators=(',',':'),ensure_ascii=False,allow_nan=False).encode()).hexdigest()

def context_hash(context):
    value={key:context.get(key) for key in ['units','graph','populations','source_hashes']}
    value.update({key:context[key] for key in ['subdivisions','demographics','geometry'] if context.get(key) is not None})
    return canonical_hash(value)

def checked_observables(plan,context,manifest,case,requested_seed):
    assert manifest['seed'] is None
    assert manifest['engine_seed']==int(requested_seed)
    recipe=copy.deepcopy(manifest['native_replay_recipe'])
    assert recipe['state']['algorithm']['metis']['seed']==requested_seed
    assert recipe['original_working_directory']==str(case)
    actual_context_hash=context_hash(context)
    assert actual_context_hash==context['context_hash']==manifest['context_hash']
    recipe_digest=context['source_hashes']['native_replay_recipe']
    assert recipe_digest==plan['provenance']['source_hashes']['native_replay_recipe']
    assert len(recipe_digest)==71 and recipe_digest.startswith('sha256:') and set(recipe_digest[7:])<=set('0123456789abcdef')

    # Only this exact, already-verified controlled cwd is projected away.
    # Every other recipe/source/option and every physical context field stays.
    recipe['original_working_directory']='<declared-case-working-directory>'
    comparison_recipe_hash=canonical_hash(recipe)
    ctx=copy.deepcopy(context);provenance=copy.deepcopy(plan['provenance'])
    ctx['source_hashes']['native_replay_recipe']=comparison_recipe_hash
    ctx['context_hash']=context_hash(ctx)
    provenance['source_hashes']['native_replay_recipe']=comparison_recipe_hash
    keys=['partition_mode','seed','engine_seed','num_districts','balance_tolerance_pct','ufactor','niter','alpha_county','split_seeds','split_epsilon','area_swing','directional_lambda','seats_per_district','total_seats','electoral_system','adjacency_sha256','context_hash']
    effective={key:manifest[key] for key in keys};effective['context_hash']=ctx['context_hash']
    return {'plan':plan['plan'],'provenance':provenance,'context':ctx,'effective_manifest_fields':effective,'resolved_recipe_with_declared_cwd':recipe}
