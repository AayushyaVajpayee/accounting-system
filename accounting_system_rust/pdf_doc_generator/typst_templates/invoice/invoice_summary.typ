#import "@preview/tablex:0.0.8":tablex,cellx,vlinex,hlinex,colspanx,rowspanx

#let additional_charges_list_content()={
  for c in add_chgs{
    (c.name,c.rate)
  }
}
#let invoice_summary(data) = {

tablex(
  columns:3,
  align:center+horizon,
     fill:(col, _r) => if calc.odd(_r) and _r!=0 { luma(240) } else { white },
  auto-hlines:false,
  hlinex(),
  colspanx(3)[*invoice summary*],
  hlinex(),
  colspanx(2)[taxable amt],data.taxable_amt,
  colspanx(2)[tax amt],data.tax_amt,
  colspanx(2)[add. chrgs],data.additional_charges_amt,
  colspanx(2)[round off],data.round_off,
    hlinex(),
  colspanx(2)[*total payable amount*],data.total_payable_amount,
  hlinex()
)
}