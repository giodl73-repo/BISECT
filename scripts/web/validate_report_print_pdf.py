"""Inspect genuine browser PDFs independently of the producing DOM/worker."""
import argparse,hashlib,io,json,unicodedata,sys
from importlib.metadata import version
from pathlib import Path
from pypdf import PdfReader
from pypdf.generic import IndirectObject,DictionaryObject,ArrayObject
from PIL import Image
import pymupdf as fitz

def canonical(text):
    # Only Unicode compatibility glyphs and whitespace differ in this contract.
    return ''.join(unicodedata.normalize('NFKC',text).split())

def inspect_actions(reader):
    seen=set()
    def walk(value):
        if isinstance(value,IndirectObject):
            identity=(value.idnum,value.generation)
            if identity in seen:return
            seen.add(identity);walk(value.get_object());return
        if isinstance(value,DictionaryObject):
            assert not set(value)&{'/JS','/JavaScript','/AA','/OpenAction','/EmbeddedFiles','/RichMedia','/AcroForm'},'Active PDF content'
            assert str(value.get('/S','')) not in {'/JavaScript','/Launch','/URI','/SubmitForm','/ImportData'},'External/action PDF content'
            for child in value.values():walk(child)
        elif isinstance(value,ArrayObject):
            for child in value:walk(child)
    walk(reader.trailer)

def flatten(image):
    value=image.convert('RGBA');background=Image.new('RGBA',value.size,'white');background.alpha_composite(value);return background.convert('RGB').tobytes()

def body_text(reader,path):
    # A named generated footer is the only text added by the print profile.
    # Verify its exact number AND location before removing it for source comparison.
    document=fitz.open(path);output=[]
    for index,page in enumerate(reader.pages):
        footer=f'Page {index+1} of {len(reader.pages)}';text=page.extract_text() or ''
        assert text.count(footer)==1,('Missing/duplicated page footer',index+1)
        visual=document[index];margin=visual.get_text('text',clip=fitz.Rect(0,visual.rect.height-34,visual.rect.width,visual.rect.height))
        assert canonical(margin)==canonical(footer),('Footer outside the expected bottom margin',index+1)
        output.append(text.replace(footer,'',1))
    return ''.join(output)

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('root');args=parser.parse_args();root=Path(args.root)
    rows=json.loads((root/'pdf-cases.json').read_text(encoding='utf-8'));results=[]
    for row in rows:
        record=json.loads((root/row['record']).read_text(encoding='utf-8'))
        assert record['original_html_sha256']==hashlib.sha256((root/row['original_html']).read_bytes()).hexdigest()
        assert record['pdf_save_observed'] is False and record['selected_pdf']['content_match_verified'] is False
        expected=(root/row['expected_text']).read_text(encoding='utf-8')
        native=PdfReader(root/row['native_pdf']);restricted=PdfReader(root/row['restricted_pdf'])
        original_text=body_text(native,root/row['native_pdf'])
        restricted_text=body_text(restricted,root/row['restricted_pdf'])
        assert canonical(original_text)==canonical(expected),(row['id'],'native PDF text differs from visible source')
        assert canonical(restricted_text)==canonical(expected),(row['id'],'restricted PDF text differs from visible source')
        assert len(native.pages)==len(restricted.pages)>=row.get('min_pages',2),(row['id'],'unexpected pagination')
        if row.get('draft_expected'):
            assert 'DRAFT' in restricted_text and 'not recomputed or authenticated' in restricted_text
            assert row['table_count']>0
        else:assert canonical('Congressional Redistricting Commission Report') in canonical(restricted_text)
        assert all(canonical(heading) in canonical(restricted_text) for heading in row['headings'])
        target=(612,792) if row['settings']['paper']=='letter' else (595.276,841.89)
        if row['settings']['orientation']=='landscape':target=target[::-1]
        for page in restricted.pages:
            assert abs(float(page.mediabox.width)-target[0])<1 and abs(float(page.mediabox.height)-target[1])<1
        inspect_actions(restricted)
        image_matches=0
        if row.get('map_png'):
            original=Image.open(root/row['map_png']);pixels=flatten(original)
            for page in restricted.pages:
                for image in page.images:
                    value=image.image
                    if value.size==original.size and flatten(value)==pixels:image_matches+=1
            assert image_matches>=1,(row['id'],'embedded native PNG pixels differ or missing')
        pdf=root/row['restricted_pdf'];assert record['selected_pdf']['sha256']==hashlib.sha256(pdf.read_bytes()).hexdigest()
        # Raster every page, retain compact contact sheets and first/map pages for visual QA.
        document=fitz.open(pdf);thumbs=[]
        for number,page in enumerate(document):
            for block in page.get_text('dict')['blocks']:
                for line in block.get('lines',[]):
                    for span in line['spans']:
                        if span['text'].strip():
                            left,top,right,bottom=span['bbox']
                            assert left>=-1 and top>=-1 and right<=page.rect.width+1 and bottom<=page.rect.height+1,(row['id'],'text outside page box',span['text'])
            pix=page.get_pixmap(matrix=fitz.Matrix(0.7,0.7),alpha=False)
            image=Image.frombytes('RGB',(pix.width,pix.height),pix.samples);thumbs.append(image)
            if number==0 or page.get_images():image.save(root/(row['id']+'-page-'+str(number+1)+'.png'))
        width=max(image.width for image in thumbs);height=sum(image.height for image in thumbs)
        sheet=Image.new('RGB',(width,height),'#dddddd');y=0
        for image in thumbs:sheet.paste(image,(0,y));y+=image.height
        sheet.save(root/(row['id']+'-contact.png'))
        results.append(dict(id=row['id'],pages=len(restricted.pages),all_visible_text_preserved=True,native_restricted_text_equal=True,media_box_matches=True,numbered_bottom_margin_footers=True,no_active_or_external_actions=True,native_png_pixel_matches=image_matches,pdf_sha256=hashlib.sha256(pdf.read_bytes()).hexdigest()))
    (root/'pdf-inspection.json').write_text(json.dumps(dict(parser='pypdf',parser_version=version('pypdf'),rasterizer='PyMuPDF',rasterizer_version=version('PyMuPDF'),pillow_version=version('Pillow'),python_version=sys.version,text_contract='Materialized DOM ordered-list markers, verified Page n of N in bottom margin; remaining source text uses NFKC compatibility glyphs and whitespace only. No numeric/punctuation/label normalization.',rows=results),indent=2)+'\n',encoding='utf-8')
    print(json.dumps(dict(root=str(root),pdf_cases=len(results),pages=sum(row['pages'] for row in results),independent_pdf_inspection=True)))

if __name__=='__main__':main()
