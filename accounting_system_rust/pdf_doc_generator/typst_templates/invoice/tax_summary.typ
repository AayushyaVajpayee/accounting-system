#import "@preview/tablex:0.0.8": tablex, cellx,vlinex,hlinex,colspanx,rowspanx

#let tax_lines2(lines,type)={
  if lines.len()==0{
    return ();
  }
  let arr = ();
  arr.push(rowspanx(lines.len())[#type]);
  for l in lines {
    arr.push([#l.tax_slab])
    arr.push([#l.tax_amount])
  }
  return arr
}
#let tax_summary_table(tax_data)=[
#tablex(
  columns:3,
  auto-hlines:false,
  fill:(col, _r) => if calc.odd(_r) and _r!=0 and col!=0 { luma(240) } else { white },
  align:center+horizon,
  hlinex(),
  [*tax type*],[*tax slab*],[*tax amount*],
  hlinex(),
  ..tax_lines2(tax_data.igst_lines,"IGST"),
    hlinex(),
  ..tax_lines2(tax_data.cgst_lines,"CGST"),
    hlinex(),
  ..tax_lines2(tax_data.sgst_lines,"SGST"),
  hlinex(),
  colspanx(2)[*total tax amount*],[#tax_data.total_tax_amount],
  hlinex(),

)
]