use pdfium_render::prelude::*;

trait SumUntilIndex<T> {
    fn sum_until_index(self) -> Vec<T>;
}

impl<T> SumUntilIndex<T> for Vec<T>
where
    T: std::ops::Add<Output = T> + Default + Copy,
{
    fn sum_until_index(mut self) -> Vec<T> {
        let mut curr_sum: T = T::default();
        for i in 0..self.len() {
            curr_sum = curr_sum + self[i];
            self[i] = curr_sum;
        }
        self
    }
}

pub fn merge_pdfs<'a>(
    pdfium: &'a Pdfium,
    mut pdfs: Vec<Vec<u8>>,
) -> Result<PdfDocument<'a>, PdfiumError> {
    if pdfs.is_empty() {
        return Ok(pdfium.create_new_pdf()?);
    } else if pdfs.len() == 1 {
        return pdfium.load_pdf_from_byte_vec(pdfs.remove(0), None);
    }

    pdfs.into_iter()
        .map(|pdf| pdfium.load_pdf_from_byte_vec(pdf, None))
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .try_fold(pdfium.create_new_pdf()?, |mut acc, pdf| {
            acc.pages_mut().append(&pdf).map(|()| acc)
        })
}

pub fn mix_first_and_last<'a>(
    pdfium: &'a Pdfium,
    pdf: PdfDocument<'a>,
) -> Result<PdfDocument<'a>, PdfiumError> {
    assert!(pdf.pages().len() > 1);

    let mut new_doc = pdfium.create_new_pdf()?;
    let mut front_index = 0;
    let mut back_index = pdf.pages().len() - 1;
    loop {
        let new_doc_pages = new_doc.pages_mut();

        new_doc_pages.copy_page_from_document(&pdf, front_index, new_doc_pages.len())?;
        front_index += 1;
        if front_index > back_index {
            break;
        }

        new_doc_pages.copy_page_from_document(&pdf, back_index, new_doc_pages.len())?;
        back_index -= 1;
        if front_index > back_index {
            break;
        }
    }

    Ok(new_doc)
}

pub fn tile_pages<'a>(
    pages: &'a PdfPages,
    scaling_factor: f32,
) -> Result<PdfDocument<'a>, PdfiumError> {
    let movement_to_center = (1.0 - scaling_factor) / 2.0;
    let mut tiled_doc = pages.tile_into_new_document(2, 4, PdfPagePaperSize::a4().landscape())?;
    for page_num in 0..tiled_doc.pages().len() {
        let mut page = tiled_doc.pages_mut().get(page_num)?;
        let (page_width, page_height) = (page.width(), page.height());

        // Add the page separators
        let objects = page.objects_mut();
        let seperator_color = PdfColor::new(0, 0, 0, 255);
        for i in 0..5 {
            let width = page_width * (i as f32 / 4.0);
            objects.create_path_object_line(
                width,
                PdfPoints::ZERO,
                width,
                page_height,
                seperator_color,
                PdfPoints::new(1.0),
            )?;
        }
        for i in 0..3 {
            let height = page_height * (i as f32 / 2.0);
            objects.create_path_object_line(
                PdfPoints::ZERO,
                height,
                page_width,
                height,
                seperator_color,
                PdfPoints::new(1.0),
            )?;
        }

        // Scale each page to add margin for printing
        page.scale(scaling_factor, scaling_factor)?;
        page.translate(
            page.width() * movement_to_center,
            page.height() * movement_to_center,
        )?;
    }

    return Ok(tiled_doc);
}
